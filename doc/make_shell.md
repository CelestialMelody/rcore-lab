> make
>
> https://www.ruanyifeng.com/blog/2015/02/make.html
>
> https://seisman.github.io/how-to-write-makefile/overview.html
>
> shell
>
> https://blog.csdn.net/Bruce_0712/article/details/78514765
>
> https://wangdoc.com/bash/



**patsubst函数**

patsubst 函数用于模式匹配的替换，格式如下。

```bash
$(patsubst pattern,replacement,text)
```



下面的例子将文件名"x.c.c bar.c"，替换成"x.c.o bar.o"。

```bash
$(patsubst %.c,%.o,x.c.c bar.c)
```





[**foreach 函数**](https://seisman.github.io/how-to-write-makefile/functions.html#foreach)

foreach函数和别的函数非常的不一样。因为这个函数是用来做循环用的，Makefile中的foreach函数几乎是仿照于Unix标准Shell（/bin/sh）中的for语句，或是C-Shell（/bin/csh）中的foreach语句而构建的。它的语法是：

```
$(foreach <var>,<list>,<text>)
```

这个函数的意思是，把参数 `<list>` 中的单词逐一取出放到参数 `<var>` 所指定的变量中，然后再执行 `<text>` 所包含的表达式。每一次 `<text>` 会返回一个字符串，循环过程中， `<text>` 的所返回的每个字符串会以空格分隔，最后当整个循环结束时， `<text>` 所返回的每个字符串所组成的整个字符串（以空格分隔）将会是foreach函数的返回值。

所以， `<var>` 最好是一个变量名， `<list>` 可以是一个表达式，而 `<text>` 中一般会使用 `<var>` 这个参数来依次枚举 `<list>` 中的单词。举个例子：

```
names := a b c d

files := $(foreach n,$(names),$(n).o)
```

上面的例子中， `$(name)` 中的单词会被挨个取出，并存到变量 `n` 中， `$(n).o` 每次根据 `$(n)` 计算出一个值，这些值以空格分隔，最后作为foreach函数的返回，所以， `$(files)` 的值是 `a.o b.o c.o d.o` 。

注意，foreach中的 `<var>` 参数是一个临时的局部变量，foreach函数执行完后，参数 `<var>` 的变量将不在作用，其作用域只在foreach函数当中。



**目标**（target）

一个目标（target）就构成一条规则。目标通常是文件名，指明Make命令所要构建的对象，比如上文的 a.txt 。目标可以是一个文件名，也可以是多个文件名，之间用空格分隔。

除了文件名，目标还可以是某个操作的名字，这称为"伪目标"（phony target）。

```
clean:
      rm *.o
```

上面代码的目标是clean，它不是文件名，而是一个操作的名字，属于"伪目标 "，作用是删除对象文件。

```
$ make  clean
```

但是，如果当前目录中，正好有一个文件叫做clean，那么这个命令不会执行。因为Make发现clean文件已经存在，就认为没有必要重新构建了，就不会执行指定的rm命令。

为了避免这种情况，可以明确声明clean是"**伪目标**"，写法如下。

```
.PHONY: clean
clean:
        rm *.o temp
```

声明clean是"伪目标"之后，make就不会去检查是否存在一个叫做clean的文件，而是每次运行都执行对应的命令。像.PHONY这样的内置目标名还有不少，可以查看[手册](https://www.gnu.org/software/make/manual/html_node/Special-Targets.html#Special-Targets)。

如果Make命令运行时没有指定目标，默认会执行Makefile文件的第一个目标。

```
$ make
```

上面代码执行Makefile文件的第一个目标。



[**filter-out**](https://seisman.github.io/how-to-write-makefile/functions.html#filter-out)

```
$(filter-out <pattern...>,<text>)
```

- 名称：反过滤函数

- 功能：以 `<pattern>` 模式过滤 `<text>` 字符串中的单词，去除符合模式 `<pattern>` 的单词。可以有多个模式。

- 返回：返回不符合模式 `<pattern>` 的字串。

- 示例：

  ```makefile
  objects=main1.o foo.o main2.o bar.o
  mains=main1.o main2.o
  ```

  `$(filter-out $(mains),$(objects))` 返回值是 `foo.o bar.o` 。



**if/then/elif/else/fi**

和C语言类似，在Shell中用`if`、`then`、`elif`、`else`、`fi`这几条命令实现分支控制。这种流程控制语句本质上也是由若干条Shell命令组成的，例如先前讲过的

```
if [ -f ~/.bashrc ]; then
    . ~/.bashrc
fi
```

其实是三条命令，`if [ -f ~/.bashrc ]`是第一条，`then .~/.bashrc`是第二条，`fi`是第三条。

如果两条命令写在同一行则需要用 ; 号隔开，一行只写一条命令就不需要写 ; 号了。

另外，`then`后面有换行，但这条命令没写完，Shell会自动续行，把下一行接在`then`后面当作一条命令处理。

和`[`命令一样，要注意命令和各参数之间必须用空格隔开。

`if`命令的参数组成一条子命令，如果该子命令的ExitStatus为0（表示真），则执行`then`后面的子命令，如果ExitStatus非0（表示假），则执行`elif`、`else`或者`fi`后面的子命令。

`if`后面的子命令通常是测试命令，但也可以是其它命令。

Shell脚本没有{}括号，所以用`fi`表示`if`语句块的结束。见下例：

```
#! /bin/sh

if [ -f /bin/bash ]
then echo "/bin/bash is a file"
else echo "/bin/bash is NOT a file"
fi
if :; then echo "always true"; fi
```

`:`是一个特殊的命令，称为空命令，该命令不做任何事，但ExitStatus总是真。此外，也可以执行`/bin/true`或`/bin/false`得到真或假的ExitStatus。

再看一个例子：

```
#! /bin/sh

echo "Is it morning? Please answer yes or no."
read YES_OR_NO
if [ "$YES_OR_NO" = "yes" ]; then
  echo "Good morning!"
elif [ "$YES_OR_NO" = "no" ]; then
  echo "Good afternoon!"
else
  echo "Sorry, $YES_OR_NO not recognized. Enter yes or no."
  exit 1
fi
exit 0
```

上例中的`read`命令的作用是等待用户输入一行字符串，将该字符串存到一个Shell变量中。

此外，Shell还提供了&&和||语法，和C语言类似，具有Short-circuit特性，很多Shell脚本喜欢写成这样：

```
test "$(whoami)" != 'root' && (echo you are using a non-privileged account; exit 1)
```

&& 相当于“if...then...”，而 || 相当于“ifnot...then...”。&& 和 || 用于连接两个命令，而上面讲的`-a`和`-o`仅用于在测试表达式中连接两个测试条件，要注意它们的区别，例如：

```
test "$VAR" -gt 1 -a "$VAR" -lt 3
```

和以下写法是等价的

```
test "$VAR" -gt 1 && test "$VAR" -lt 3
```
