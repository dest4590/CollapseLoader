import os
import sys
from collections import OrderedDict

import yaml

try:
    current_script_dir = os.path.dirname(__file__)

    if not current_script_dir:
        current_script_dir = os.getcwd()
    collapse_folder = os.path.abspath(os.path.join(current_script_dir, ".."))

    if not os.path.exists(os.path.join(collapse_folder, "collapse")):
        collapse_folder = os.path.abspath(
            os.path.join(os.path.dirname(__file__), "..", "..", "..")
        )

    LANG_FOLDER = os.path.join(collapse_folder, "collapse", "assets", "lang")

except NameError:
    collapse_folder = os.path.abspath(os.path.join(os.getcwd(), ".."))
    LANG_FOLDER = os.path.join(collapse_folder, "collapse", "assets", "lang")


def represent_ordereddict(dumper, data):
    value = []
    for item_key, item_value in data.items():
        node_key = dumper.represent_data(item_key)
        node_value = dumper.represent_data(item_value)
        value.append((node_key, node_value))
    return yaml.nodes.MappingNode("tag:yaml.org,2002:map", value)


yaml.add_representer(OrderedDict, represent_ordereddict)


def load_yaml_ordered(stream):
    class OrderedLoader(yaml.SafeLoader):
        pass

    def construct_mapping(loader, node):
        loader.flatten_mapping(node)
        return OrderedDict(loader.construct_pairs(node))

    OrderedLoader.add_constructor(
        yaml.resolver.BaseResolver.DEFAULT_MAPPING_TAG, construct_mapping
    )
    return yaml.load(stream, Loader=OrderedLoader)


def get_nested_value(data_dict, key_string, default=None):
    keys = key_string.split(".")
    value = data_dict
    try:
        for key in keys:
            if isinstance(value, dict):
                value = value.get(key)
                if value is None:
                    return default
            else:
                return default
        return value
    except (KeyError, TypeError):
        return default


def set_nested_value(data_dict, key_string, value):
    keys = key_string.split(".")
    current_level = data_dict
    for i, key in enumerate(keys):
        if i == len(keys) - 1:

            current_level[key] = value
        else:

            if key not in current_level or not isinstance(current_level[key], dict):
                current_level[key] = OrderedDict()
            current_level = current_level[key]


print(f"Attempting to use language folder: {LANG_FOLDER}")

if not os.path.isdir(LANG_FOLDER):
    print(f"Error: Language folder not found at '{LANG_FOLDER}'")
    print("Please check the LANG_FOLDER path calculation in the script.")
    sys.exit(1)

try:
    lang_files = [f for f in os.listdir(LANG_FOLDER) if f.endswith(".yml")]
    languages = sorted([f.split(".")[0] for f in lang_files])
except Exception as e:
    print(f"Error reading language folder '{LANG_FOLDER}': {e}")
    sys.exit(1)

if not languages:
    print(f"Error: No language (.yml) files found in '{LANG_FOLDER}'")
    sys.exit(1)

print(f"Found languages: {', '.join(languages)}")
print("-" * 30)

while True:
    print("\nEnter the translation key (e.g., 'menu.return', 'api.error')")
    key_to_edit = input("Key (or press Enter to exit): ").strip()

    if not key_to_edit:
        print("Exiting.")
        break

    new_translations = OrderedDict()
    loaded_data = OrderedDict()

    print("-" * 30)
    print(f"Editing key: '{key_to_edit}'")
    print("-" * 30)

    valid_input_received = False
    for lang_code in languages:
        lang_file_path = os.path.join(LANG_FOLDER, f"{lang_code}.yml")
        current_value_str = "[File not found or invalid]"
        current_data = None

        try:
            with open(lang_file_path, "r", encoding="utf-8") as f:
                current_data = load_yaml_ordered(f)
                loaded_data[lang_code] = current_data
                current_value = get_nested_value(current_data, key_to_edit)
                if current_value is not None:

                    if isinstance(current_value, str) and "\n" in current_value:
                        current_value_str = (
                            f"[Current multi-line value]\n{current_value}\n---"
                        )
                    else:
                        current_value_str = f"Current: '{current_value}'"

                else:
                    current_value_str = "[Key not found]"

        except FileNotFoundError:
            loaded_data[lang_code] = OrderedDict()
            current_value_str = "[File not found]"
        except yaml.YAMLError as e:
            print(f"Error reading YAML for language '{lang_code}': {e}")
            current_value_str = "[YAML Read Error]"

            loaded_data[lang_code] = OrderedDict()
        except Exception as e:
            print(f"An unexpected error occurred reading '{lang_code}.yml': {e}")
            current_value_str = "[Read Error]"
            loaded_data[lang_code] = OrderedDict()

        print(f"\nLanguage: [{lang_code}] - {current_value_str}")

        print("Enter new value (Type '!!' on a new line to finish multi-line input):")
        lines = []
        while True:
            try:
                line = input("> ")
                if line.strip() == "!!":
                    break
                lines.append(line)
            except EOFError:
                break
        new_value = "\n".join(lines)

        new_translations[lang_code] = new_value
        valid_input_received = True

    if not valid_input_received:
        print("No valid inputs received. Skipping save.")
        continue

    print("\n" + "=" * 30)
    print("Summary of changes:")
    for lang_code, value in new_translations.items():

        display_value = value.replace("\n", "\\n")
        if len(display_value) > 60:
            display_value = display_value[:57] + "..."
        print(f"  {lang_code}: '{display_value}'")

    print("=" * 30)

    confirm = input("Save these changes? (y/N): ").strip().lower()

    if confirm == "y":

        print("Saving changes...")
        saved_count = 0
        error_count = 0
        for lang_code, data_to_save in loaded_data.items():
            if data_to_save is None:
                print(f"Skipping save for '{lang_code}' due to previous load error.")
                error_count += 1
                continue

            lang_file_path = os.path.join(LANG_FOLDER, f"{lang_code}.yml")
            try:

                value_to_set = new_translations.get(lang_code, "")

                set_nested_value(data_to_save, key_to_edit, value_to_set)

                with open(lang_file_path, "w", encoding="utf-8") as f:
                    yaml.dump(
                        data_to_save, f, allow_unicode=True, sort_keys=False, indent=4
                    )

                saved_count += 1
            except yaml.YAMLError as e:
                print(f"Error writing YAML for language '{lang_code}': {e}")
                error_count += 1
            except IOError as e:
                print(f"Error writing file '{lang_file_path}': {e}")
                error_count += 1
            except Exception as e:
                print(f"An unexpected error occurred saving '{lang_code}.yml': {e}")
                error_count += 1

        print(f"\nSave complete. {saved_count} files updated, {error_count} errors.")
    else:
        print("Changes discarded.")

    print("-" * 30)
