import json
import os
import shutil
from datetime import datetime

from ...storage.Data import data
from ...utils.Fixes import console
from ..Language import lang
from ..Module import Module
from .Client import Client


class CustomClient(Client):
    """Custom client class for user-added clients"""

    def __init__(
        self,
        name: str,
        jar_path: str,
        version: str = "1.12.2",
        main_class: str = "net.minecraft.client.main.Main",
        is_fabric: bool = False,
        custom_id: int = 0,
    ) -> None:
        filename = os.path.basename(jar_path)
        link = os.path.join(data.root_dir, filename)

        super().__init__(
            name=name,
            link=link,
            main_class=main_class,
            version=version,
            internal=False,
            working=True,
            id=custom_id + 10000,
            fabric=is_fabric,
        )

        self.original_jar_path = jar_path
        self.custom_id = custom_id
        self.added_date = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        self.is_custom = True

    def to_dict(self) -> dict:
        """Convert to dictionary for storage"""
        return {
            "name": self.name,
            "version": self.version,
            "main_class": self.main_class,
            "fabric": self.fabric,
            "original_jar_path": self.original_jar_path,
            "filename": self.filename,
            "custom_id": self.custom_id,
            "added_date": self.added_date,
        }

    @classmethod
    def from_dict(cls, data_dict: dict) -> "CustomClient":
        """Create a custom client from a dictionary"""
        return cls(
            name=data_dict["name"],
            jar_path=data_dict["original_jar_path"],
            version=data_dict["version"],
            main_class=data_dict["main_class"],
            is_fabric=data_dict["fabric"],
            custom_id=data_dict["custom_id"],
        )


class CustomClientManager(Module):
    """Manager for user-added custom clients"""

    def __init__(self) -> None:
        super().__init__()
        self.clients = []
        self.config_path = os.path.join(data.root_dir, "custom_clients.json")
        self.load_clients()

    def load_clients(self) -> None:
        """Load custom clients from the JSON file"""
        if os.path.exists(self.config_path):
            try:
                with open(self.config_path, "r", encoding="utf-8") as f:
                    clients_data = json.load(f)

                self.clients = [
                    CustomClient.from_dict(client_data) for client_data in clients_data
                ]
                self.debug(lang.t("customclients.loaded").format(len(self.clients)))
            except Exception as e:
                self.error(lang.t("customclients.load-failed").format(e))
                self.clients = []

    def save_clients(self) -> None:
        """Save custom clients to the JSON file"""
        try:
            with open(self.config_path, "w", encoding="utf-8") as f:
                json.dump([client.to_dict() for client in self.clients], f, indent=4)
            self.debug(lang.t("customclients.saved"))
        except Exception as e:
            self.error(lang.t("customclients.save-failed").format(e))

    def add_client(
        self,
        jar_path: str,
        name: str = None,
        version: str = "1.12.2",
        is_fabric: bool = False,
        main_class: str = "net.minecraft.client.main.Main",
    ) -> CustomClient:
        """Add a new custom client from a JAR file path"""
        if not os.path.exists(jar_path):
            self.error(lang.t("customclients.jar-not-found").format(jar_path))
            return None

        if not name:
            basename = os.path.basename(jar_path)
            name = os.path.splitext(basename)[0]

        custom_id = 1
        if self.clients:
            custom_id = max(client.custom_id for client in self.clients) + 1

        client = CustomClient(
            name=name,
            jar_path=jar_path,
            version=version,
            is_fabric=is_fabric,
            custom_id=custom_id,
            main_class=main_class,
        )

        destination = os.path.join(data.root_dir, os.path.basename(jar_path))
        try:
            os.makedirs(client.path_dir, exist_ok=True)

            if not os.path.isfile(destination):
                self.info(lang.t("customclients.copying").format(jar_path, destination))
                shutil.copy2(jar_path, destination)
            else:
                self.debug(lang.t("customclients.already-exists").format(jar_path))
        except Exception as e:
            self.error(f"Failed to copy JAR file: {e}")
            return None

        self.clients.append(client)
        self.save_clients()
        self.info(lang.t("customclients.added").format(name))
        return client

    def remove_client(self, client_id: int) -> bool:
        """Remove a custom client by ID"""
        for i, client in enumerate(self.clients):
            if client.custom_id == client_id:
                client_name = client.name
                if os.path.isdir(client.path_dir):
                    shutil.rmtree(client.path_dir, ignore_errors=True)

                self.clients.pop(i)
                self.save_clients()
                self.info(lang.t("customclients.removed").format(client_name))
                return True

        self.error(lang.t("customclients.not-found").format(client_id))
        return False

    def rename_client(self, client_id: int, new_name: str) -> bool:
        """Rename a custom client"""
        for client in self.clients:
            if client.custom_id == client_id:
                old_name = client.name
                client.name = new_name
                self.save_clients()
                self.info(lang.t("customclients.renamed").format(old_name, new_name))
                return True

        self.error(lang.t("customclients.not-found").format(client_id))
        return False

    def update_version(self, client_id: int, new_version: str) -> bool:
        """Update the version of a custom client"""
        for client in self.clients:
            if client.custom_id == client_id:
                old_version = client.version
                client.version = new_version
                self.save_clients()
                self.info(
                    lang.t("customclients.version-changed").format(
                        client.name, old_version, new_version
                    )
                )
                return True

        self.error(lang.t("customclients.not-found").format(client_id))
        return False

    def get_client_by_id(self, client_id: int) -> CustomClient:
        """Get a custom client by ID"""
        for client in self.clients:
            if client.custom_id == client_id:
                return client
        return None

    def get_client_by_name(self, name: str) -> CustomClient:
        """Get a custom client by name"""
        for client in self.clients:
            if name.lower() in client.name.lower():
                return client
        return None

    def show_client_manager(self) -> None:
        """Display the custom client manager menu"""
        from ...render.CLI import selector

        selector.set_title(title_type="customclients")
        menu_option_offset = 10

        while True:
            console.print(f"[bold]{lang.t("customclients.menu-header")}[/]")

            if not self.clients:
                console.print(f"[yellow]{lang.t('customclients.no-clients')}[/]")
            else:
                for i, client in enumerate(self.clients):
                    print(
                        f"{i + 1}. {client.name} <{client.version}> {'(Fabric)' if client.fabric else ''}"
                    )

            client_count = len(self.clients)
            console.print(f"\n{lang.t('customclients.menu-options')}")
            console.print(
                f"[dark_cyan]{client_count + menu_option_offset + 1}. {lang.t('customclients.add')}[/]"
            )
            console.print(
                f"[dark_cyan]{client_count + menu_option_offset + 2}. {lang.t('customclients.remove')}[/]"
            )
            console.print(
                f"[dark_cyan]{client_count + menu_option_offset + 3}. {lang.t('customclients.rename')}[/]"
            )
            console.print(
                f"[dark_cyan]{client_count + menu_option_offset + 4}. {lang.t('customclients.change-version')}[/]"
            )
            console.print(
                f"[dark_cyan]{client_count + menu_option_offset + 5}. {lang.t('customclients.back')}[/]"
            )

            try:
                choice = selector.ask_int(lang.t("cli.select"))
                menu_start = client_count + menu_option_offset + 1

                if choice == menu_start:
                    jar_path = input(f"{lang.t('customclients.enter-jar-path')}: ")

                    if not os.path.exists(jar_path):
                        selector.warn(
                            lang.t("customclients.jar-not-found").format(jar_path)
                        )
                        continue

                    name = input(
                        f"{lang.t('customclients.enter-name')} ({os.path.splitext(os.path.basename(jar_path))[0]}): "
                    )
                    if not name:
                        name = os.path.splitext(os.path.basename(jar_path))[0]

                    version = input(
                        f"{lang.t('customclients.enter-version')} (1.12.2): "
                    )
                    if not version:
                        version = "1.12.2"

                    main_class = input(
                        f"{lang.t('customclients.enter-main-class')} (net.minecraft.client.main.Main): "
                    )
                    if not main_class:
                        main_class = "net.minecraft.client.main.Main"

                    is_fabric = selector.ask(lang.t("customclients.is-fabric"))

                    self.add_client(jar_path, name, version, is_fabric, main_class)

                elif choice == menu_start + 1:
                    if not self.clients:
                        selector.warn(lang.t("customclients.no-clients"))
                        continue

                    client_index = selector.ask_int(
                        lang.t("customclients.select-client")
                    )

                    if 1 <= client_index <= len(self.clients):
                        client = self.clients[client_index - 1]
                        if selector.ask(
                            lang.t("customclients.confirm-remove").format(client.name)
                        ):
                            self.remove_client(client.custom_id)
                    else:
                        selector.warn(lang.t("customclients.invalid-choice"))

                elif choice == menu_start + 2:
                    if not self.clients:
                        selector.warn(lang.t("customclients.no-clients"))
                        continue

                    client_index = selector.ask_int(
                        lang.t("customclients.select-client")
                    )

                    if 1 <= client_index <= len(self.clients):
                        client = self.clients[client_index - 1]
                        new_name = input(
                            f"{lang.t('customclients.enter-new-name')} ({client.name}): "
                        )

                        if new_name:
                            self.rename_client(client.custom_id, new_name)
                    else:
                        selector.warn(lang.t("customclients.invalid-choice"))

                elif choice == menu_start + 3:
                    if not self.clients:
                        selector.warn(lang.t("customclients.no-clients"))
                        continue

                    client_index = selector.ask_int(
                        lang.t("customclients.select-client")
                    )

                    if 1 <= client_index <= len(self.clients):
                        client = self.clients[client_index - 1]
                        new_version = input(
                            f"{lang.t('customclients.enter-new-version')} ({client.version}): "
                        )

                        if new_version:
                            self.update_version(client.custom_id, new_version)
                    else:
                        selector.warn(lang.t("customclients.invalid-choice"))

                elif choice == menu_start + 4:
                    break

                else:
                    if 1 <= choice <= client_count:
                        client = self.clients[choice - 1]

                        print(
                            f"\n{lang.t('customclients.client-options').format(client.name)}"
                        )
                        console.print(f"[dark_cyan]1. {lang.t('customclients.run')}[/]")
                        console.print(
                            f"[dark_cyan]2. {lang.t('customclients.open-folder')}[/]"
                        )
                        console.print(
                            f"[dark_cyan]3. {lang.t('customclients.rename')}[/]"
                        )
                        console.print(
                            f"[dark_cyan]4. {lang.t('customclients.change-version')}[/]"
                        )
                        console.print(
                            f"[dark_cyan]5. {lang.t('customclients.remove')}[/]"
                        )
                        console.print(
                            f"[dark_cyan]6. {lang.t('customclients.back')}[/]"
                        )

                        sub_choice = selector.select()

                        if sub_choice == "1":
                            client.run()
                        elif sub_choice == "2":
                            if not client.open_folder():
                                selector.warn(
                                    lang.t("main.client-not-installed").format(
                                        client.name
                                    )
                                )
                        elif sub_choice == "3":
                            new_name = input(
                                f"{lang.t('customclients.enter-new-name')} ({client.name}): "
                            )
                            if new_name:
                                self.rename_client(client.custom_id, new_name)
                        elif sub_choice == "4":
                            new_version = input(
                                f"{lang.t('customclients.enter-new-version')} ({client.version}): "
                            )
                            if new_version:
                                self.update_version(client.custom_id, new_version)
                        elif sub_choice == "5":
                            if selector.ask(
                                lang.t("customclients.confirm-remove").format(
                                    client.name
                                )
                            ):
                                self.remove_client(client.custom_id)

            except Exception as e:
                self.error(f"Error in custom client manager: {e}")
                selector.warn(str(e))
                # uncomment to see the full traceback
                # selector.warn(f"{e}\n{traceback.format_exc()}")

        selector.reset_title()


custom_client_manager = CustomClientManager()
