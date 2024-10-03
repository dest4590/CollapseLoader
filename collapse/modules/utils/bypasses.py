import os
import sys
from colorama import Fore
import string
import random
import subprocess





class colors:
    green = "\033[92m"
    red = "\033[91m"
    yellow = "\033[93m"
    blue = "\033[94m"
    purple = "\033[95m"
    cyan = "\033[96m"
    reset = "\033[0m"

def clearConsole():
    clr = lambda: os.system('cls' if os.name in ('nt', 'dos') else 'clear')
    return clr


def echo_bypass():#
    current_directory = os.path.dirname(os.path.abspath(__file__))
    echoless_jar_path = os.path.join(current_directory, 'bypass', 'Echoless.jar')
    print("[BYPASS] Launching Echoless.jar...")
    try:
        if not os.path.isfile(echoless_jar_path):
            print(f"{colors.red}[BYPASS] Echoless.jar not found at {echoless_jar_path}!{colors.reset}")
            return False
        subprocess.call(['java', '-jar', echoless_jar_path])
        print(f"{colors.green}[BYPASS] Echo.ac bypass successfully executed!{colors.reset}")
        return True  # Erfolgreiche Ausführung
    except subprocess.CalledProcessError as e:
        print(f"{colors.red}[BYPASS] Failed to run Echoless.jar! Error: {e}{colors.reset}")
    except Exception as e:
        print(f"{colors.red}[BYPASS] An unexpected error occurred: {e}{colors.reset}")
    return False  # Bei Misserfolg




def bypass():
    print(f"{colors.purple}[BYPASS] Start Bypassing..")
    N = 15
    tit = ''.join(random.choices(string.ascii_uppercase +
                                 string.digits, k=N))
    os.system(f"title {tit}")
    subprocess.call(['ipconfig', '/flushdns'])
    clearConsole()
    print("[BYPASS] Basic Bypass Loaded, Run echo.ac bypass for more!")





