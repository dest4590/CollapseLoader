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

from .modules.render.CLI import selector
from .modules.render.Header import header
from .modules.render.menus.CreditsMenu import credits_menu
from .modules.storage.Data import data
from .modules.storage.Settings import settings
from .modules.utils.clients.ClientManager import client_manager
from .modules.utils.Language import lang


def initialize_settings():
    from .modules.storage.Options import Option

    if not settings.get("nickname"):
        logger.warning(lang.t("main.nickname-reminder"))
    Option("nickname").create(f"Collapse{random.randint(1000, 9999)}")
    Option("rpc").create(True, "Loader")
    Option("read_messages").create("0,", "Loader")


def display_main_menu():
    from .modules.utils.Logo import logo

    text = ""

    if settings.use_option("hide_logo"):
        logo_type = logo.full if settings.use_option("use_short_logo") else logo.short
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


def handle_selection(choosed: str):
    try:
        choosed_int = int(choosed)
        client_index_max = len(client_manager.clients)

        if 1 <= choosed_int <= client_index_max:
            client = selector.get_client_by_index(choosed_int)
            client.run()
            return
        elif choosed_int > client_index_max:
            handle_menu_options(choosed_int)
            return

    except ValueError:
        handle_commands(choosed)


def handle_commands(command_str: str):
    """Handles commands"""
    try:
        args = selector.parse_args(command_str.lower())
        command_name = args[0]

        if not command_name:
            raise ValueError("Empty command")

        for command in commands:
            if command.cmd == command_name:
                try:
                    client_arg = args[1]
                    try:
                        client = selector.get_client_by_index(int(client_arg))
                    except ValueError:
                        client = selector.get_client_by_name(client_arg)

                    command.execute(client, args[2:])
                    return
                except (IndexError, ValueError):
                    command.execute(None, args[1:])
                    return

        selector.warn(lang.t("main.invalid-option"))

    except (ValueError, IndexError):
        selector.warn(lang.t("main.invalid-option"))


def handle_menu_options(choosed_int: int):
    offset = selector.offset
    choosed_int -= offset

    if choosed_int == 11:
        from .modules.storage.Options import options_menu

        options_menu.show()
    elif choosed_int == 12:
        from .modules.network.Configs import config_menu

        config_menu.show()
    elif choosed_int == 13:
        settings.set("nickname", selector.select_username())
        logger.debug(lang.t("main.nickname-changed"))
    elif choosed_int == 14:
        settings.set("ram", selector.ask_int(lang.t("main.select-ram")) * 1024)
        logger.debug(lang.t("main.ram-changed"))
    elif choosed_int == 15:
        from .modules.storage.ClientCleaner import clientcleaner

        clientcleaner.scan_folders()
    elif choosed_int == 16:
        credits_menu.show()
    elif choosed_int == 17:
        sys.exit(1)
    else:
        selector.warn(lang.t("main.invalid-option"))


def main():

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
