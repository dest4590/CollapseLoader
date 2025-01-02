import os
from subprocess import Popen

from rich import box
from rich.markup import escape
from rich.table import Table

from ..render.CLI import selector
from ..storage.Cache import cache
from ..storage.Data import data
from ..utils.clients.Client import Client
from ..utils.clients.ClientManager import client_manager
from ..utils.Fixes import console
from .Language import lang


class Command:
    def __init__(
        self,
        cmd: str,
        description_key: str,
        usage: str,
        execution: callable,
        requires_client: bool = True,
    ):
        self.cmd = cmd
        self.description = lang.t(description_key)
        self.usage = usage
        self.execution = execution
        self.requires_client = requires_client

    def __str__(self):
        return self.cmd

    def execute(self, client=None, args=None):
        self.execution(client, args)


def open_data_folder(*args):
    Popen(f"explorer /open,{os.path.abspath(data.root_dir)}")


def handle_client_command(client: Client, args, action):
    if client is None:
        selector.warn(lang.t("main.client-not-found").format(args[0] if args else "?"))
        return

    if action == "download":
        client.download()
    elif action == "reset":
        client.reset()
        selector.warn(lang.t("main.client-resetted").format(client.name))
    elif action == "delete":
        client.delete()
        selector.warn(lang.t("main.client-deleted").format(client.name))
        selector.refresh_text()
    elif action == "open":
        if not client.open_folder():
            selector.warn(lang.t("main.client-not-installed").format(client.name))


def handle_crashes():
    crash_logs_path = data.get_local("crash_logs")
    if os.path.isdir(crash_logs_path):
        Popen(f"explorer /open,{os.path.abspath(crash_logs_path)}")
    else:
        selector.warn(lang.t("main.no-crash-logs"))


def handle_data_clear(client: Client, args):
    if args == ["clear"]:
        data.clear()
        selector.warn(lang.t("main.data-cleared"))
    else:
        open_data_folder()


def handle_cache(args):
    if args == ["clear"]:
        cache.clear()
        selector.warn(lang.t("cache.cache-cleared"))
    elif args == ["create"]:
        cache.save(client_manager.json_clients)
        selector.pause()
    elif args == ["info"]:
        cache.display_info()
        selector.pause()


def display_help(*args):
    print()

    table = Table(title=lang.t("commands.title"), box=box.ROUNDED)
    table.add_column(lang.t("commands.command"))
    table.add_column(lang.t("commands.description"))
    table.add_column(lang.t("commands.usage"))

    for command in commands:
        table.add_row(command.cmd, command.description, command.usage)

    console.print(table)

    selector.pause()


commands = [
    Command(
        "crashes",
        "commands.cmds.crashes",
        "crashes",
        lambda clients, args: handle_crashes(),
        requires_client=False,
    ),
    Command(
        "data",
        "commands.cmds.data",
        escape("data [clear]"),
        lambda client, args: handle_data_clear(client, args),
        requires_client=False,
    ),
    Command(
        "cache",
        "commands.cmds.cache",
        escape("cache [clear, create, info]"),
        lambda client, args: handle_cache(args),
        requires_client=False,
    ),
    Command(
        "download",
        "commands.cmds.download",
        "download <name/number>",
        lambda client, args: handle_client_command(client, args, "download"),
    ),
    Command(
        "reset",
        "commands.cmds.reset",
        "reset <name/number>",
        lambda client, args: handle_client_command(client, args, "reset"),
    ),
    Command(
        "delete",
        "commands.cmds.delete",
        "delete <name/number>",
        lambda client, args: handle_client_command(client, args, "delete"),
    ),
    Command(
        "open",
        "commands.cmds.open",
        "open <name/number>",
        lambda client, args: handle_client_command(client, args, "open"),
    ),
    Command(
        "help",
        "commands.cmds.help",
        "help",
        display_help,
        requires_client=False,
    ),
]
