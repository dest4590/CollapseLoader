import argparse

parser = argparse.ArgumentParser(description='Your script description')
parser.add_argument('-v', action='store_true', help='Enable debug logging')
parser.add_argument('--disable-analytics', action='store_true', help='Disable analytics')
parser.add_argument('--server', action='store_true', help='Run the SDK server')

args, unknown = parser.parse_known_args()