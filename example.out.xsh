#!/usr/bin/env xonsh

p"~/.xonshrc".exists() && source ~/.xonshrc

from glob import glob
from os.path import dirname,abspath,join,basename


PWD = dirname(abspath(__file__))
cd @(PWD)

for file in glob(join(PWD,"examples/*.rs")):
  name = basename(file)[:-3]
  cargo run --example @(name) > examples/@(name).out




