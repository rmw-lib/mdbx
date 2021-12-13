#!/usr/bin/env xonsh

$RAISE_SUBPROC_ERROR = True
$XONSH_SHOW_TRACEBACK = True

from os.path import dirname,abspath,exists,join

DIR = dirname(abspath(__file__))
cd @(DIR)

npx md-padding -i ./readme.make.md

npx @rmw/md-include .markdown.json

out = "../blog-vuepress2/site/zh/log/2021-12-21-mdbx.md"

with open("README.md") as f:
  md = f.read()
  md = md[md.find("#"):md.find("## 关于")]
  with open(out,"w") as o:
      o.write(md)



