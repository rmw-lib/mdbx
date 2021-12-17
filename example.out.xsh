#!/usr/bin/env xonsh

p"~/.xonshrc".exists() && source ~/.xonshrc


from glob import glob
from os.path import dirname,abspath,join,basename
PWD = dirname(abspath(__file__))

cd @(PWD)/git/example/01
cargo run > main.out

cd @(PWD)

for file in glob(join(PWD,"examples/*.rs")):
  name = basename(file)[:-3]
  out = f"examples/{name}.out"
  cargo run --example @(name) > @(out) 2>&1
  head -5 @(out)
  sed -i '1,3d' @(out)


./md.xsh
