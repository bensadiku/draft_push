#!/usr/bin/env bash

draft_push_path=home/$(whoami)

git config --global alias.xpush '!git push $1 $2 && ../.././draft_push'