import logging
import os
import random
import sys
from subprocess import Popen

from .arguments import args
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

from .modules.storage.Data import data  # isort: skip
from .modules.storage.Settings import settings  # isort: skip
from .modules.utils.Language import lang  # isort: skip
from .modules.render.CLI import selector
from .modules.render.Header import header
from .modules.render.menus.CreditsMenu import credits_menu
from .modules.utils.clients.ClientManager import client_manager


def initialize_settings() -> None:
    """Initialize user settings with default values if not already set"""
    from .modules.storage.Options import Option

    if not settings.get("nickname"):
        logger.warning(lang.t("main.nickname-reminder"))

    Option("nickname").create(f"Collapse{random.randint(1000, 9999)}")
    Option("rpc").create(True, "Loader")
    Option("read_messages").create("0,", "Loader")


def display_main_menu() -> None:
    """Display the main menu with logo and options"""
    from .modules.utils.Logo import logo

    text = ""

    if settings.use_option("hide_logo"):
        logo_type = logo.full if settings.use_option("use_short_logo") else logo.short
        selector.animate(f"[bold white]{logo_type}\n", highlight=False)

        text += f"\t [italic]Version: {data.version}[/] ([steel_blue3]{data.codename.upper()}[/])\n"
        text += f"[bold green]{logo.tagline}[/]\n"

        if settings.use_option("hide_links"):
            text += "[slate_blue3]Discord: https://collapseloader.org/discord\n"
            text += "[dodger_blue1]Telegram: https://collapseloader.org/telegram"

    if selector.header is not None and not settings.use_option("show_header"):
        text += f"\n\n{selector.header}"

    selector.animate(text)
    selector.show()


def handle_selection(choosed: str) -> None:
    """Handle the user's menu selection"""
    if choosed.isnumeric():
        if int(choosed) <= len(client_manager.clients):
            client = selector.get_client_by_index(int(choosed))
            client.run()
            return

    else:
        try:
            args = selector.parse_args(choosed)
            args[0] = args[0].lower()

            if args[0] == "":
                raise ValueError

            try:
                if args[1].isnumeric():
                    client = selector.get_client_by_index(int(args[1]))
                else:
                    client = selector.get_client_by_name(args[1])
            except (IndexError, ValueError):
                if args[0] == "l":
                    if os.path.isdir(data.get_local("crash_logs")):
                        absolute_path = os.path.abspath(data.get_local("crash_logs"))
                        Popen(f"explorer /open,{absolute_path}")
                    else:
                        selector.warn(lang.t("main.no-crash-logs"))
                    return

                selector.warn(lang.t("main.select-client"))
                return

            if client is None:
                selector.warn(lang.t("main.client-not-found").format(args[1]))
                return

            if args[0] == "r":
                client.reset()
                selector.warn(lang.t("main.client-resetted").format(client.name))

            elif args[0] == "d":
                client = selector.get_client_by_index(int(args[1]))
                client.delete()
                selector.warn(lang.t("main.client-deleted").format(client.name))
                selector.refresh_text()

            elif args[0] == "o":
                client = selector.get_client_by_index(int(args[1]))
                client.open_folder()

        except ValueError:
            selector.warn(lang.t("main.invalid-option"))
        return

    choosed = int(choosed)

    if choosed == selector.offset + 11:
        from .modules.storage.Options import options_menu

        options_menu.show()
    elif choosed == selector.offset + 12:
        from .modules.network.Configs import config_menu

        config_menu.show()
    elif choosed == selector.offset + 13:
        settings.set("nickname", selector.select_username())
        logger.debug(lang.t("main.nickname-changed"))
    elif choosed == selector.offset + 14:
        settings.set("ram", selector.ask_int(lang.t("main.select-ram")) * 1024)
        logger.debug(lang.t("main.ram-changed"))
    elif choosed == selector.offset + 15:
        from .modules.storage.ClientCleaner import clientcleaner
        clientcleaner.scan_folders()
    elif choosed == selector.offset + 16:
        credits_menu.show()
    elif choosed == selector.offset + 17:
        sys.exit(1)
    else:
        selector.warn(lang.t("main.invalid-option"))


def main() -> None:
    """Main function to run the loader"""
    if "_child.py" not in sys.argv[0]:
        from .modules.network.Analytics import analytics
        from .modules.network.Message import messages
        from .modules.network.Updater import updater
        from .modules.sdk.SdkServer import server
        from .modules.utils.RPC import rpc

        initialize_settings()

        updater.check_version()
        analytics.loader_start()

        if args.server:
            server.run(port=args.port if args.port else 9090)

        if args.crash:
            raise Exception(lang.t("main.force-crash"))

        else:
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
