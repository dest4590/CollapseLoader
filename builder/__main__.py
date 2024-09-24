from argparse import ArgumentParser

from .builder import Builder

parser = ArgumentParser(description='CollapseLoader builder')
parser.add_argument('--dev', action='store_true', help='Build the dev version of the loader')

args, unknown = parser.parse_known_args()

Builder(name='CollapseLoader', dev=args.dev).build()
