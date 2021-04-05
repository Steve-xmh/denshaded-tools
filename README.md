# Densha De D Tools

## Chinese - 简体中文

### 简介

一个用于解/打包基于 [Selene/Lue](http://selene-lue.halfmoon.jp/) 引擎的 電車でＤ 系列游戏的游戏数据包的工具。

通过解包后将文件夹与数据包名称一致并放在游戏文件夹内，可以对游戏资源进行修改。

但是由于逆向中一个数据的计算方式缺失，故打包出来的数据无法被加载，还请有人能够分析出这段参数的意义。

### 用法

```bash
# 打印帮助信息
denshaded-tools help

# 解包
# 将 ./file/to/game.Pack 解包到 ./file/to/game
denshaded-tools unpack ./file/to/game.Pack
# 将 ./file/to/game.Pack 解包到 ./any/dir
denshaded-tools unpack ./file/to/game.Pack -o ./any/dir

# 打包（暂无法使用）
# 将 ./file/to/game 打包到 ./file/to/game.Pack
denshaded-tools pack ./file/to/game
# 将 ./file/to/game 打包到 ./any/file.Pack
denshaded-tools pack ./file/to/game -o ./any/file.Pack
```
