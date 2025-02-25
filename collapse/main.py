import logging
import random
import sys

from .arguments import args
from .modules.utils.Commands import commands
from .modules.utils.Fixes import console
from .modules.utils.Logger import logger

if args.v:
    logger.setLevel(logging.DEBUG)

if args.level:
    levels = {
        "debug": logging.DEBUG,
        "info": logging.INFO,
        "warning": logging.WARNING,
        "error": logging.ERROR,
        "critical": logging.CRITICAL,
    }
    logger.setLevel(levels[args.level])

from .modules.network.Analytics import analytics
from .modules.network.Configs import config_menu
from .modules.network.Message import messages
from .modules.network.Updater import updater
from .modules.render.CLI import selector, selector_offset
from .modules.render.Header import header
from .modules.render.menus.CreditsMenu import credits_menu
from .modules.sdk.SdkServer import server
from .modules.storage.ClientCleaner import clientcleaner
from .modules.storage.Data import data
from .modules.storage.Options import Option, options_menu
from .modules.storage.Settings import settings
from .modules.utils.clients.ClientManager import client_manager
from .modules.utils.Language import lang
from .modules.utils.Logo import logo
from .modules.utils.RPC import rpc


def initialize_settings() -> None:
    """Initializes default settings if they don't exist."""
    if not settings.get("nickname"):
        logger.warning(lang.t("main.nickname-reminder"))
        settings.set("nickname", f"Collapse{random.randint(1000, 9999)}")

    Option("rpc").create(True, "Loader")
    Option("read_messages").create("0,", "Loader")


def display_main_menu() -> None:
    """Displays the main menu to the user."""
    text = ""

    if settings.use_option("hide_logo"):
        logo_type = logo.short if settings.use_option("use_short_logo") else logo.full
        console.print(f"[bold white]{logo_type}\n", highlight=False)
        text += f"\t [italic]Version: {data.version}[/] ([steel_blue3]{data.codename.upper()}[/])\n"
        text += f"[bold green]{logo.tagline}[/]\n"

        if settings.use_option("hide_links"):
            text += "[slate_blue3]Discord: https://collapseloader.org/discord\n"
            text += "[dodger_blue1]Telegram: https://collapseloader.org/telegram"

    if selector.header is not None and not settings.use_option("show_header"):
        text += f"\n\n{selector.header}"

    console.print(text)
    selector.show()


MENU_OPTION_OFFSET = 10


def handle_selection(choosed: str) -> None:
    """Handles the user's selection from the main menu."""

    try:
        choosed_int = int(choosed)
        client_index_max = len(client_manager.clients)
        menu_start = client_index_max + MENU_OPTION_OFFSET
        menu_end = client_index_max + selector_offset

        if 1 <= choosed_int <= client_index_max:
            client = selector.get_client_by_index(choosed_int)
            client.run()
        elif menu_start <= choosed_int <= menu_end:
            handle_menu_options(choosed_int)
        else:
            selector.warn(lang.t("main.invalid-option"))

    except ValueError:
        handle_commands(choosed)
    except Exception as e:
        selector.warn(f"An unexpected error occurred: {e}")
        logger.exception("An unexpected error during selection handling:")


def handle_commands(command_str: str) -> None:
    """Handles commands using a command dictionary."""
    try:
        args = selector.parse_args(command_str.lower())
        command_name = args[0]
        if not command_name:
            raise ValueError("Empty command")

        command_map = {cmd.cmd: cmd for cmd in commands}
        for cmd in commands:
            for alias in cmd.aliases:
                command_map[alias] = cmd

        command = command_map.get(command_name)
        if command:
            try:
                client = None
                if command.requires_client:
                    client_arg = args[1]
                    try:
                        client = selector.get_client_by_index(int(client_arg))
                    except ValueError:
                        client = selector.get_client_by_name(client_arg)
                    if client is None:
                        raise ValueError(
                            lang.t("main.invalid-client-name").format(name=client_arg)
                        )
                    command.execute(client, args[2:])
                else:
                    command.execute(None, args[1:])

            except IndexError:
                selector.warn(
                    lang.t("main.missing-arguments").format(usage=command.usage)
                )
            except ValueError as e:
                selector.warn(str(e))
            except Exception as e:
                selector.warn(
                    f"An unexpected error occurred while executing command: {e}"
                )
                logger.exception(f"Error executing command '{command_name}':")
        else:
            selector.warn(lang.t("main.invalid-option"))

    except ValueError as e:
        if "Empty command" in str(e):
            selector.warn(lang.t("main.empty-command"))
        else:
            selector.warn(str(e))
    except Exception as e:
        selector.warn(f"An unexpected error occurred: {e}")
        logger.exception("An unexpected error in handle_commands:")


def handle_menu_options(choosed_int: int) -> None:
    """Handles menu options using a dictionary."""
    menu_actions = {
        selector_offset: options_menu.show,
        selector_offset + 1: config_menu.show,
        selector_offset
        + 2: lambda: settings.set("nickname", selector.select_username())
        or logger.debug(lang.t("main.nickname-changed")),
        selector_offset
        + 3: lambda: settings.set(
            "ram", selector.ask_int(lang.t("main.select-ram")) * 1024
        )
        or logger.debug(lang.t("main.ram-changed")),
        selector_offset + 4: clientcleaner.scan_folders,
        selector_offset + 5: credits_menu.show,
        selector_offset + 6: lambda: sys.exit(1),
    }

    action = menu_actions.get(choosed_int)
    if action:
        try:
            action()
        except Exception as e:
            selector.warn(f"An unexpected error occurred in menu option: {e}")
            logger.exception(f"Error in menu option {choosed_int}:")
    else:
        selector.warn(lang.t("main.invalid-option"))
        logger.warning(f"Invalid menu option selected: {choosed_int}")


def main() -> None:
    """Main entry point of the application."""
    selector.offset = 0
    if "_child.py" not in sys.argv[0]:
        initialize_settings()

        updater.check_version()
        analytics.loader_start()

        if args.server:
            server.run(port=args.port if args.port else 9090)

        if args.crash:
            raise Exception(lang.t("main.force-crash"))

        selector.set_title(selector.titles_states["default"])
        rpc.start()

        if settings.get("language_setup") is None:
            lang.setup_language()
            settings.set("language_setup", True)
            selector.refresh_text()
            header.get()

        while True:
            display_main_menu()

            if not messages.shown:
                messages.show_messages()

            choosed = selector.select()
            handle_selection(choosed)
