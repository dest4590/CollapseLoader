import os
from subprocess import Popen

from rich import box
from rich.table import Table

from ..render.CLI import selector
from ..storage.Data import data
from ..utils.clients.Client import Client
from ..utils.Fixes import console
from .Language import lang


class Command:
    def __init__(self, cmd: str, description_key: str, execution: callable):
        self.cmd = cmd
        self.description_key = description_key
        self.execution = execution

    def __str__(self):
        return self.cmd

    def get_description(self):
        return lang.t(self.description_key)

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


def display_help(*args):
    print()

    table = Table(title=lang.t("commands.title"), box=box.ROUNDED)
    table.add_column(lang.t("commands.command"))
    table.add_column(lang.t("commands.description"))

    for command in commands:
        table.add_row(command.cmd, command.get_description())

    console.print(table)

    selector.pause()


commands = [
    Command("crashes", "commands.crashes", lambda: handle_crashes),
    Command(
        "data",
        "commands.data",
        handle_data_clear,
    ),
    Command(
        "download",
        "commands.download",
        lambda client, args: handle_client_command(client, args, "download"),
    ),
    Command(
        "reset",
        "commands.reset",
        lambda client, args: handle_client_command(client, args, "reset"),
    ),
    Command(
        "delete",
        "commands.delete",
        lambda client, args: handle_client_command(client, args, "delete"),
    ),
    Command(
        "open",
        "commands.open",
        lambda client, args: handle_client_command(client, args, "open"),
    ),
    Command("help", "commands.help", display_help),
]
