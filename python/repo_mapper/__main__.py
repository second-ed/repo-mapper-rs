import argparse
import os
import sys
import repo_mapper_py

if __name__ == "__main__":

    def str_to_list(inp_str: str) -> list[str]:
        return inp_str.split(",")

    parser = argparse.ArgumentParser()

    parser.add_argument(
        "--repo-root",
        type=os.path.abspath,
        required=True,
        help="Path to the root of the repo to generate the map for.",
    )
    parser.add_argument(
        "--readme-path",
        type=os.path.abspath,
        required=True,
        help="Path to the readme file to add the map to.",
    )
    parser.add_argument(
        "--gitignore-path",
        type=os.path.abspath,
        required=True,
        help="Path to the .gitignore.",
    )
    parser.add_argument(
        "--allowed-exts",
        default=["py", "md", "toml", "lock", "yaml", "ipynb"],
        type=str_to_list,
        help="A comma separated string of extensions to remove. E.g. 'py,rs,toml'. Defaults to: 'py,md,toml,lock,yaml,ipynb'",
    )
    parser.add_argument(
        "--ignore-dirs",
        default=[],
        type=str_to_list,
        help="A comma separated string of directories to ignore. E.g. '.venv,target'.",
    )
    parser.add_argument(
        "--ignore-hidden",
        action="store_true",
        help="Flag to ignore hidden files. E.g. those that start with a '.' like '.env'.",
    )
    parser.add_argument(
        "--dirs-only",
        action="store_true",
        help="Flag to only map directories instead of files",
    )
    args = parser.parse_args()
    sys.exit(
        int(
            repo_mapper_py.py_main(
                repo_root=args.repo_root,
                readme_path=args.readme_path,
                gitignore_path=args.gitignore_path,
                allowed_exts=args.allowed_exts,
                ignore_dirs=args.ignore_dirs,
                ignore_hidden=args.ignore_hidden,
                dirs_only=args.dirs_only,
            )
        )
    )
