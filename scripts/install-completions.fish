#!/usr/bin/env fish

set -l root (realpath (status dirname)/..)
set -l completions $root/completions

if contains -- --regenerate $argv
    if not command -q home
        echo "home not found on PATH, cannot regenerate" >&2
        exit 1
    end

    set -l generated (mktemp)
    command home completions fish >$generated
    and mv $generated $completions/home.fish
    or begin
        rm -f $generated
        exit 1
    end

    echo "regenerated $completions/home.fish"
end

if not command -q jq
    echo "warning: jq not found, dynamic id completions will be empty" >&2
end

set -l target $__fish_config_dir/completions
mkdir -p $target

cat $completions/home.fish $completions/home-dynamic.fish >$target/home.fish
or exit 1

echo "installed completions to $target/home.fish"
