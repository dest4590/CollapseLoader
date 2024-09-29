import argparse

parser = argparse.ArgumentParser(description='Your script description')
parser.add_argument('-v', action='store_true', help='Enable debug logging')
parser.add_argument('--disable-analytics', action='store_true', help='Disable analytics')
parser.add_argument('--server', action='store_true', help='Run the SDK server')
parser.add_argument('--port', type=int, help='Set port for SDK server')
parser.add_argument('--timeout', type=int, help='Set timeout for network requests')
parser.add_argument('--crash', action='store_true', help='Force crash')
parser.add_argument('--api-url', help='Set API URL')

args, unknown = parser.parse_known_args()