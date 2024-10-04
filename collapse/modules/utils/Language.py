import os
import sys

import yaml

from ...arguments import args
from ...constants import LANGUAGES
from ..storage.Settings import settings
from ..utils.Module import Module


class Language(Module):
    def __init__(self):
        super().__init__()
        language = settings.get('language')

        try:
            self.base_path = sys._MEIPASS
        except AttributeError:
            self.base_path = os.path.abspath(".")

        self.languages = LANGUAGES
        
        if language is None:
            language = 'en'
            settings.set('language', language)

        if args.lang:
            language = args.lang
            settings.set('language', language)

        if not os.path.join(self.base_path, 'collapse', 'assets', 'lang', f'{language}.yml'):
            self.error(f'Language file not found: {language}.yml, set default language: en')
            self.set_language('en')

        self.current = settings.get('language')
        self.translations = self.load_language_file(self.current)
    
    def load_language_file(self, lang_code):

        file_path = os.path.join(self.base_path, 'collapse', 'assets', 'lang', f'{lang_code}.yml')

        with open(file_path, 'r', encoding='utf-8') as file:
            return yaml.safe_load(file)

    def set_language(self, lang_code):
        self.translations = self.load_language_file(lang_code)
        settings.set('language', lang_code)

    def setup_language(self):
        while True:
            lang = input(f'Select language ({', '.join(self.languages)}): ')
            
            if lang in self.languages:
                self.set_language(lang)
                break
            else:
                self.error('Select language!')
                os.system('pause')
                pass

    def t(self, key) -> str:
        keys = key.split('.')
        value = self.translations
        for k in keys:
            value = value.get(k, None)
            if value is None:
                return key
        return value

lang = Language()