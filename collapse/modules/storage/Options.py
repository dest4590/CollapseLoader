import random
from typing import List

from rich import print

from ..render.CLI import console, selector
from ..utils.Language import lang
from ..utils.Logger import logger
from ..utils.Module import Module
from .Settings import settings

option_list: List["Option"] = []


class Option(Module):
    """The Option class represents a configurable option."""

    def __init__(
        self,
        name: str,
        description: str = "",
        option_type=str,
        default_value=object,
        callback=None,
        highlight: bool = False,
    ) -> None:
        super().__init__(False)
        self.name = name
        self.description = description
        self.option_type = option_type
        self.value = settings.get(name)
        self.default_value = default_value
        self.callback = callback
        self.highlight = highlight

        if description:
            option_list.append(self)

    @property
    def line(self) -> str:
        """Returns a formatted string representing the option"""
        self.value = settings.get(self.name)
        if self.option_type == bool:
            self.value = (
                f'[green]{lang.t("options.enabled")}[/]'
                if self.value == "True"
                else f'[red]{lang.t("options.disabled")}[/]'
            )
        return f"{self.name.title().replace('_', ' ')}[/] / [light_salmon3]{self.description}[/] * {self.value}"

    def create(self, value=None, header: str = "Options") -> None:
        """Creates a new option in the settings"""
        if not settings.get(self.name, header):
            settings.set(
                self.name, value if value is not None else self.default_value, header
            )
            self.debug(
                lang.t("options.created").format(
                    self.name,
                    value if value is not None else self.default_value,
                    header,
                )
            )

    def save(self, value: object) -> None:
        """Saves option to settings file"""
        settings.set(self.name, value)
        self.info(lang.t("options.saved").format(self.name, value))
        if self.callback:
            if callable(self.callback):
                self.callback()

    def input(self) -> None:
        """Handles user input for the option"""
        if self.option_type == str:
            console.print(f"\n{lang.t('options.input.note')}")
            new_value = console.input(
                lang.t("options.input.prompt").format(self.name, self.value)
            )

            if new_value != "":
                self.save(
                    new_value if new_value.upper() != "RESET" else self.default_value
                )

        elif self.option_type == bool:
            current_value = settings.get(self.name)
            self.save(not current_value.lower() == "true")

        elif self.option_type == int:
            ram_map = {
                "2G": 2048,
                "4G": 4096,
                "8G": 8192,
                "16G": 16384,
            }

            console.print(f'\n{lang.t("options.input.note-ram")}')
            new_value = console.input(
                lang.t("options.input.prompt").format(self.name, self.value)
            )

            if new_value != "":
                new_value_upper = new_value.upper()
                if new_value_upper in list(ram_map.keys()):
                    self.save(ram_map[new_value_upper])
                    return

                try:
                    self.save(int(new_value))
                except ValueError:
                    logger.error(lang.t("options.invalid-value"))
                    selector.pause()

    def reset(self) -> None:
        """Reset option with default value"""
        self.save(self.default_value)
        self.debug(lang.t("options.reset").format(self.name, self.default_value))

    @staticmethod
    def get_option_by_index(index: int) -> "Option":
        """Gets the option by its index"""
        return option_list[index]

    def __str__(self):
        """Returns option name as title"""
        return self.name.title().replace("_", " ")


# fmt: off
general_options = [
    Option("nickname", lang.t("options.settings.nickname"), default_value=f"Collapse{random.randint(1000, 9999)}", highlight=True),
    Option("hide_messages", lang.t("options.settings.hide_messages"), bool, False),
    Option("hide_links", lang.t("options.settings.hide_links"), bool, False),
    Option("show_client_version", lang.t("options.settings.show_client_version"), bool, False),
    Option("discord_rpc", lang.t("options.settings.discord_rich_presence"), bool, True),
    Option("hide_console", lang.t("options.settings.hide_console"), bool, False),
    Option("show_installed", lang.t("options.settings.show_installed"), bool, False),
    Option("language", lang.t("options.settings.language").format(", ".join(lang.languages)), default_value="en"),
    Option("sort_clients", lang.t("options.settings.sort_clients"), bool, False),
]

performance_options = [
    Option("ram", lang.t("options.settings.ram"), int, 2048),
    Option("disable_caching", lang.t("options.settings.disable_caching"), bool, False),
]

appearance_options = [
    Option("custom_title", lang.t("options.settings.custom_title"), default_value="None"),
    Option("hide_logo", lang.t("options.settings.hide_logo"), bool, False),
    Option("use_short_logo", lang.t("options.settings.use_short_logo"), bool, False),
    Option("show_header", lang.t("options.settings.show_header"), bool, True),
]
# fmt: on

for opt_list in [general_options, performance_options, appearance_options]:
    for option in opt_list:
        option.create()

categorized_options = [
    (lang.t("options.category.general"), general_options),
    (lang.t("options.category.performance"), performance_options),
    (lang.t("options.category.appearance"), appearance_options),
]

option_list = (
    general_options + performance_options + appearance_options
)  # still need this for reset


class Menu:
    """Options menu"""

    def __init__(self) -> None:
        pass

    def _get_option(self, choice: int) -> Option | None:
        """Helper function to get the option."""
        current_index = 1
        for _, options in categorized_options:
            for option in options:
                if current_index == choice:
                    return option
                current_index += 1
        return None

    def show(self) -> None:
        """Displays the options menu"""
        selector.set_title(title_type="settings")

        while True:
            print()
            option_lines = []
            current_index = 1

            for category_name, options in categorized_options:
                option_lines.append(f"[bold]{category_name}[/]")
                for option in options:
                    option_lines.append(
                        f'[{"green" if not option.highlight else "green3"}]{current_index}. {option.line}'
                    )
                    current_index += 1

            option_lines.append(
                f'[dark_red]{current_index}. {lang.t("menu.return")}[/]'
            )
            return_index = current_index
            current_index += 1
            option_lines.append(
                f'[bright_red]{current_index}. {lang.t("options.reset-all")}[/]'
            )
            reset_all_index = current_index

            console.print("\n".join(option_lines), highlight=False)

            try:
                choice = int(console.input(f"{lang.t('options.choose')}: "))

                if choice == return_index:
                    break
                elif choice == reset_all_index:
                    if selector.ask(lang.t("options.ask-reset")):
                        for option in option_list:
                            option.reset()
                    continue

                selected_option = self._get_option(choice)
                if selected_option:
                    selected_option.input()
                else:
                    logger.error(lang.t("options.invalid-choice"))

            except ValueError:
                logger.error(lang.t("options.invalid-choice"))
            except Exception as e:
                logger.error(f"An unexpected error occurred: {e}")

        selector.reset_title()


options_menu = Menu()
