import os
import sys

import yaml

from ...arguments import args
from ..storage.Settings import settings


class Language:
    def __init__(self):
        language = settings.get('language')
        
        if language is None:
            language = 'en'
            settings.set('language', language)

        if args.lang:
            language = args.lang
            settings.set('language', language)

        self.translations = self.load_language_file(settings.get('language'))
    
    def load_language_file(self, lang_code):
        try:
            base_path = sys._MEIPASS
        except AttributeError:
            base_path = os.path.abspath(".")

        file_path = os.path.join(base_path, 'collapse', 'assets', 'lang', f'{lang_code}.yml')

        with open(file_path, 'r', encoding='utf-8') as file:
            return yaml.safe_load(file)

    def t(self, key) -> str:
        keys = key.split('.')
        value = self.translations
        for k in keys:
            value = value.get(k, None)
            if value is None:
                return key
        return value

lang = Language()