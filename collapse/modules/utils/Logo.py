from ..utils.Language import lang


class Logo:
    """Just logos"""

    full = r'''
   ___         _  _                          
  / __\  ___  | || |  ____  ____   ___   ___ 
 / /    / _ \ | || | / _  ||  _ \ / __| / _ \
/ /___ | (_) || || || (_| || |_) |\__ \|  __/
\____/  \___/ |_||_| \____||  __/ |___/ \___|
                           |_|               '''

    short = r'''
   ___    __  
  / __\  / /  
 / /    / /   
/ /___ / /___ 
\____/ \____/ 
              '''

    tagline = lang.t('logo.tagline')

logo = Logo()