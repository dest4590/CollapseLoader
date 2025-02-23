import os
from abc import ABC, abstractmethod
from subprocess import Popen
from typing import List, Optional

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


class Command(ABC):
    """Abstract base class for commands."""

    def __init__(
        self,
        cmd: str,
        description_key: str,
        usage: str,
        requires_client: bool = True,
        aliases: List[str] = None,
    ):
        self.cmd = cmd
        self.description = lang.t(description_key)
        self.usage = usage
        self.requires_client = requires_client
        self.aliases = aliases or []

    @abstractmethod
    def execute(self, client: Optional[Client], args: List[str]):
        """Execute the command.

        Args:
            client: The active client (if required), or None.
            args: The arguments passed to the command.
        """
        pass

    def __str__(self):
        return self.cmd


class OpenDataFolderCommand(Command):
    def __init__(self):
        super().__init__(
            "datafolder",
            "commands.cmds.datafolder",
            "datafolder",
            requires_client=False,
            aliases=["opendata"],
        )

    def execute(self, client: Optional[Client], args: List[str]):
        Popen(f"explorer /open,{os.path.abspath(data.root_dir)}")


class CrashesCommand(Command):
    def __init__(self):
        super().__init__(
            "crashes", "commands.cmds.crashes", "crashes", requires_client=False
        )

    def execute(self, client: Optional[Client], args: List[str]):
        crash_logs_path = data.get_local("crash_logs")
        if os.path.isdir(crash_logs_path):
            Popen(f"explorer /open,{os.path.abspath(crash_logs_path)}")
        else:
            selector.warn(lang.t("main.no-crash-logs"))


class DataCommand(Command):
    def __init__(self):
        super().__init__(
            "data", "commands.cmds.data", "data [clear]", requires_client=False
        )

    def execute(self, client: Optional[Client], args: List[str]):
        if args == ["clear"]:
            data.clear()
            selector.warn(lang.t("main.data-cleared"))
        else:
            OpenDataFolderCommand().execute(None, [])


class CacheCommand(Command):
    def __init__(self):
        super().__init__(
            "cache",
            "commands.cmds.cache",
            "cache [clear, create, info]",
            requires_client=False,
        )

    def execute(self, client: Optional[Client], args: List[str]):
        if args == ["clear"]:
            cache.clear()
            selector.warn(lang.t("cache.cache-cleared"))
        elif args == ["create"]:
            cache.save(client_manager.json_clients)
            selector.pause()
        elif args == ["info"]:
            cache.display_info()
            selector.pause()


class ClientCommand(Command):
    def __init__(
        self,
        cmd: str,
        description_key: str,
        usage: str,
        action: str,
        aliases: List[str] = None,
    ):
        super().__init__(cmd, description_key, usage, aliases=aliases)
        self.action = action

    def execute(self, client: Optional[Client], args: List[str]):
        if client is None:
            selector.warn(
                lang.t("main.client-not-found").format(args[0] if args else "?")
            )
            return
        if self.action == "download":
            client.download()
        elif self.action == "reset":
            client.reset()
            selector.warn(lang.t("main.client-resetted").format(client.name))
        elif self.action == "delete":
            client.delete()
            selector.warn(lang.t("main.client-deleted").format(client.name))
            selector.refresh_text()
        elif self.action == "open":
            if not client.open_folder():
                selector.warn(lang.t("main.client-not-installed").format(client.name))


class DownloadCommand(ClientCommand):
    def __init__(self):
        super().__init__(
            "download",
            "commands.cmds.download",
            "download <name/number>",
            "download",
            aliases=["dl"],
        )


class ResetCommand(ClientCommand):
    def __init__(self):
        super().__init__("reset", "commands.cmds.reset", "reset <name/number>", "reset")


class DeleteCommand(ClientCommand):
    def __init__(self):
        super().__init__(
            "delete",
            "commands.cmds.delete",
            "delete <name/number>",
            "delete",
            aliases=["del"],
        )


class OpenCommand(ClientCommand):
    def __init__(self):
        super().__init__("open", "commands.cmds.open", "open <name/number>", "open")


class HelpCommand(Command):
    def __init__(self):
        super().__init__(
            "help",
            "commands.cmds.help",
            "help",
            requires_client=False,
            aliases=["h", "?"],
        )

    def execute(self, client: Optional[Client], args: List[str]):
        print()

        table = Table(title=lang.t("commands.title"), box=box.ROUNDED)
        table.add_column(lang.t("commands.command"))
        table.add_column(lang.t("commands.description"))
        table.add_column(lang.t("commands.usage"))

        for command in commands:
            names = ", ".join([command.cmd] + command.aliases)
            table.add_row(names, command.description, escape(command.usage))

        console.print(table)
        selector.pause()


commands: List[Command] = []


def register_command(command: Command):
    """Registers a command in the global command list."""
    commands.append(command)


register_command(DownloadCommand())
register_command(ResetCommand())
register_command(DeleteCommand())
register_command(OpenCommand())
register_command(CrashesCommand())
register_command(DataCommand())
register_command(CacheCommand())
register_command(HelpCommand())
register_command(OpenDataFolderCommand())
