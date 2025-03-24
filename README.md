# VnpyRS —— 与vnpy兼容但更快的回测框架

VnpyRS是以提升性能为目的，部分代码用Rust语言重新实现的vnpy，实现了vnpy的回测与参数调优功能。

它有三种工作模式：
- 脚本运行模式
- 图形界面运行模式
- vnpy插件模式（将极速K线图表嵌入vnpy为其提供服务）

具体快多少？
- K线图表快几个数量级，你没听错，说快100倍可能都太保守，K线超过一定数量后vnpy会完全卡死，而VnpyRS依然流畅
- 数据库读取速度最快是vnpy的6倍以上（Sqlite）
- 策略代码执行快7%-87%，具体见下表：
（测试环境：Win11，Python3.11.3，CPU:12th Gen Intel(R) Core(TM) i7-12700H   2.30 GHz，内存:32G。
测试时均用GUI且置于前台，测多次取次小值。
另：不同的Python版本性能优化程度不同，其中Python通用版是最慢的，做性能测试时不应考虑该版本）

|策略名称                | vnpy耗时 | VnpyRS耗时 | 加速百分比 |
|-----------------------|----------|-----------|---------|
|AtrRsiStrategy         | 11.7     | 8.62  | 36% |
|BollChannelStrategy    | 1.82     | 1.09  | 67% |
|DoubleMaStrategy       | 5.04     | 4.45  | 13% |
|DualThrustStrategy     | 6.38     | 3.42  | 87% |
|KingKeltnerStrategy    | 2.86     | 1.92  | 49% |
|MultiSignalStrategy    | 12.1     | 11.3  | 7%  |
|MultiTimeframeStrategy | 2.47     | 1.55  | 59% |
|TurtleSignalStrategy   | 19.8     | 11.8  | 68% |


## 由来

众所周知，Python生态强大，编码灵活，但是有个缺点，就是慢。2024年，我实验性的用Rust重写了vnpy的回测模块，惊喜的发现，运行速度提升了近20倍。
显然，移植到Rust这件事的价值是巨大的。但还有个问题，由于它100%是用Rust编写的，包括策略也是用Rust编写的，那用户就无法使用Python的各种库了。况且，Rust是出了名的难入门的语言。上述的两点会降低它的实用性。

因此，在2025年我重新设定了目标，该项目必须完全兼容Python生态，理想情况下100%兼容已经为vnpy编写的策略，同时要提升性能。
由于用户代码是用Python编写的，那么性能提升就不会像用纯Rust编写时那样夸张了，但省下的时间依然不少。如果用examples文件夹里面的案例测试，在不改一行策略代码的情况下，在10核32G的电脑上，配合Python3.11，综合速度提升了一倍。（具体提升多少涉及硬件配置、Python版本和具体策略）

## 开发路线图

不会完全移植vnpy。一个原因是vnpy用到了很多动态语言的特性，以实现插件化，这部分在静态语言上实现很困难，或者需要大量的unsafe代码。另一个原因是没必要，涉及网络的模块，绝大部分时间花在网络的等待上，即使通过Rust、C++重写，体验也不会明显变好。

未来可能会支持用Rust、C、C++写策略，这样的话回测性能提升10倍也是有可能的，但需要谨慎的思考如何设计接口。

## 环境准备

VnpyRS对python包的依赖和vnpy几乎一样。Python版本需要3.7以上，推荐3.10以上

## pip安装

最佳的安装方式是源码编译安装，这样可以让pyo3启用针对特定的Python版本进行性能优化，而且大部分的深度vnpy用户都对代码做过修改，VnpyRS的代码基本上和vnpy原版的代码一一对应，这样您可以把对vnpy的修改移植到VnpyRS上去（虽然需要学习Rust，但是在有本工程源码作为范例的情况下不会很难）。

如果只是想试用一下，或者仅仅使用vnpy插件模式，可以通过pip安装，但PYPI服务器上的版本一般是Python通用版本，未针对特定Python版本做过性能优化（影响回测性能，但不影响K线图表的性能，后者完全是用Rust写的）。

**Windows**

```
pip install vnpyrs
```

**Linux**

```
pip install vnpyrs
```

**Macos**

目前Mac需要源码编译安装

## 源码安装

**Windows**

以下以Windows 11为例

1.安装Git并拉取源代码
```
git clone https://github.com/vnpyrs/vnpyrs.git
```
若网络不畅，可以从gitee拉取
```
git clone https://gitee.com/vnpyrs/vnpyrs.git
```

2.安装rust

先从Rust的官网下载64位的rustup-init.exe：
`https://www.rust-lang.org/zh-CN/tools/install`

双击rustup-init.exe进行安装，需要做出选择时按1进行快速安装，期间会安装Visual Studio社区版，需要手动进行，因为Rust依赖MSVC的开发环境，该步骤是必要的（WSL用户和已安装Visual Studio的用户除外）。

安装完成后打开cmd控制台键入“cargo”以验证是否安装成功

接下来设置crates.io的镜像，以加快下载Rust依赖包的速度

创建文件C:/Users/你的用户名/.cargo/config，内容如下：
```
[source.crates-io]
replace-with = 'rsproxy-sparse'
[source.rsproxy]
registry = "https://rsproxy.cn/crates.io-index"
[source.rsproxy-sparse]
registry = "sparse+https://rsproxy.cn/index/"
[registries.rsproxy]
index = "https://rsproxy.cn/crates.io-index"
[net]
git-fetch-with-cli = true
```
注意此文件的后缀并非.txt，如果看不到后缀，需要进入文件夹选项，去掉“隐藏已知文件类型的扩展名”前面的勾，之后再删除.txt后缀。

3.安装Python和maturin

登录Python的官网 https://www.python.org/ ，下载所需版本的Python。也可以选择Anaconda之类的环境，但我们以官方Python为例。可采用默认选项安装，但建议添加环境变量。

VnpyRS是一个Rust和Python的混合项目，因此还需要安装maturin插件，您可以全局安装
```
pip install maturin
```
或者在创建Python虚拟环境后安装，创建虚拟环境的问题后面单独讲，Python社区更加建议用此方式。而且如果想修改VnpyRS代码的话，使用虚拟环境会更加容易

4.把项目编译成whl文件

切换到项目的根目录下（就是README.md所在的文件夹），输入：
```
python -m maturin build -r
```
现在项目根目录下会多出一个target文件夹，里面有个wheels文件夹，再里面有一个.whl后缀的文件，这个就是针对您当前环境的Python版本做性能优化过的包。如果您之前可以正常运行vnpy，请跳过第5步，直接进行第6步

5.安装ta-lib

从这里下载ta-lib Python版的whl包：https://github.com/cgohlke/talib-build/releases ，并通过pip安装，命令为：
```
pip install (whl文件的文件名)
```
例如：`pip install ta_lib-0.6.3-cp313-cp313-win_amd64.whl`

注意安装过程中会下载它所依赖的Python包，建议使用pip代理以加快下载速度，设置方法自行搜索

6.安装VnpyRS的whl文件

```
pip install (whl文件的文件名)
```
例如：`pip install target\wheels\vnpyrs-0.2.0-cp313-cp313-win_amd64.whl`

至此安装完成

7.（可选）建立Python虚拟环境，并以调试模式编译、运行VnpyRS

在项目根目录下执行以下命令
```
python -m venv .env
```
此时会多出来一个.env文件夹。cd进“.env\Scripts”文件夹，再执行“activate”进入该虚拟环境，再cd回来“cd ../..”。
之后再运行pip、python3、maturin命令的话，只影响该环境，或被该环境影响。直到退出shell会话

这个时候您无需运行“python3 -m maturin build -r”、“pip install (whl文件的文件名)”两条命令编译VnpyRS，而只需要一条：
```
python3 -m maturin develop -r
```
这个技巧在需要修改VnpyRS代码的时候特别有用


**Linux**

以下以Ubuntu为例

1.拉取源代码
```
git clone https://github.com/vnpyrs/vnpyrs.git
```
若网络不畅，可以从gitee拉取
```
git clone https://gitee.com/vnpyrs/vnpyrs.git
```

2.安装rust

Rust的官网网站是https://www.rust-lang.org/ ，但因为官方的下载速度越来越慢，建议用国内字节的代理，代理的官网是https://rsproxy.cn/
```
export RUSTUP_DIST_SERVER="https://rsproxy.cn"
export RUSTUP_UPDATE_ROOT="https://rsproxy.cn/rustup"
curl --proto '=https' --tlsv1.2 -sSf https://rsproxy.cn/rustup-init.sh | sh
```
要求选择时直接回车就行

安装完成后退出shell，重新登陆，键入“cargo”以验证是否安装成功

接下来设置crates.io的镜像，以加快下载Rust依赖包的速度

创建文件~/.cargo/config，内容如下：

```
[source.crates-io]
replace-with = 'rsproxy-sparse'
[source.rsproxy]
registry = "https://rsproxy.cn/crates.io-index"
[source.rsproxy-sparse]
registry = "sparse+https://rsproxy.cn/index/"
[registries.rsproxy]
index = "https://rsproxy.cn/crates.io-index"
[net]
git-fetch-with-cli = true
```

3.安装maturin

VnpyRS是一个Rust和Python的混合项目，因此还需要安装maturin插件，您可以全局安装
```
pip install maturin
```
或者在创建Python虚拟环境后安装，创建虚拟环境的问题后面单独讲，Python社区更加建议用此方式。而且如果想修改VnpyRS代码的话，使用虚拟环境会更加容易

4.把项目编译成whl文件

切换到项目的根目录下（就是README.md所在的文件夹），输入：
```
python3 -m maturin build -r
```
现在项目根目录下会多出一个target文件夹，里面有个wheels文件夹，再里面有一个.whl后缀的文件，这个就是针对您当前环境的Python版本做性能优化过的包。如果您之前可以正常运行vnpy，请跳过第5步，直接进行第6步

5.安装ta-lib

如果想通过pip的方式安装Python版本的ta-lib，必须先安装C++版本的ta-lib。ta-lib的最新版本是0.6，相比之前，官方给出了更为详细的安装步骤：https://ta-lib.org/install/#linux-debian-packages 。

对于大部分人来说可以下载页面上的*_amd64.deb包并使用以下命令安装，虽然这个是C++版本的，但是安装了这个以后，Python版本的ta-lib会因依赖关系随其他包自动安装
```
sudo dpkg -i ta-lib_0.6.4_amd64.deb
```
另一种安装方式是找到对应操作系统的对应Python版本的whl包，但ta-lib官方只提供Windows平台的。

6.安装VnpyRS的whl文件

whl文件可以通过pip来安装。注意安装过程中会下载它所依赖的Python包，建议使用pip代理以加快下载速度，设置方法自行搜索
```
pip install (whl文件的文件名)
```
至此安装完成

7.（可选）建立Python虚拟环境，并以调试模式编译、运行VnpyRS

在项目根目录下执行以下命令
```
python3 -m venv .env
```
此时会多出来一个.env文件夹。执行以下命令进入该虚拟环境
```
source .env/bin/activate
```
之后再运行pip、python3、maturin命令的话，只影响该环境，或被该环境影响。直到退出shell会话

这个时候您无需运行“python3 -m maturin build -r”、“pip install (whl文件的文件名)”两条命令编译VnpyRS，而只需要一条：
```
python3 -m maturin develop -r
```
这个技巧在需要修改VnpyRS代码的时候特别有用


## 脚本运行模式

VnpyRS支持脚本运行，在任意目录下创建backtest.py，写入以下示例代码：

```Python
from vnpyrs.backtesting import BacktestingEngine, BacktestingMode
from vnpyrs.trader.optimize import OptimizationSetting

#double_ma_strategy模块是vnpy自带的，复制到当前目录即可。对策略的导入必须在导入vnpyrs之后，除非将其中的vnpy_ctastrategy替换为vnpyrs
from double_ma_strategy import DoubleMaStrategy

from datetime import datetime
import time

def main():
    engine=BacktestingEngine()

    engine.set_parameters(vt_symbol="300.LOCAL",
                        interval="1m",
                        start=datetime(2009,1,5),
                        end=datetime(2020,8,18),
                        rate=2.3e-5,
                        slippage=0.2,
                        size=1,
                        pricetick=0.2,
                        capital=10000000)

    start = time.perf_counter()
    engine.add_strategy(DoubleMaStrategy,{'fast_window': 10, 'slow_window': 20})
    engine.load_data()
    engine.run_backtesting()
    df=engine.calculate_result()
    print(df)
    engine.calculate_statistics()
    engine.show_chart().show()

    setting=OptimizationSetting()
    setting.set_target("sharpe_ratio")
    setting.add_parameter("fast_window",8,12,1)
    setting.add_parameter("slow_window",12,20,1)
    engine.run_bf_optimization(setting)
    engine.run_ga_optimization(setting)
    print(time.perf_counter()-start) #这里用于统计耗时，可以看到vnpyrs耗时比vnpy少

if __name__ == '__main__':
    main()
```

VnpyRS使用的数据库和json配置文件和vnpy完全一样，二者是共用数据库的（但部分数据库暂时不支持，会在未来补上）。标的300.LOCAL的数据在examples下，导入300_1min_vnpy.csv到vnpy引擎里即可。
在该目录下打开CMD（按住Shift->点击鼠标右键->在此处打开命令窗口/PowerShell）后运行下列命令启动VnpyRS：
```
python backtest.py
```

## 图形界面运行模式

VnpyRS还支持图形界面运行。和vnpy一样，在家目录下建立一个名为“strategies”的文件夹，在里面新建一个名为`__init__.py`的空文件，再将包含策略的py文件放到“strategies”文件夹里。

在任意目录下创建gui.py，写入以下示例代码：

```Python
from vnpyrs.widget import create_qapp, BacktesterWindow


def main():
    """"""
    qapp = create_qapp()
    backtester_window = BacktesterWindow()
    backtester_window.showMaximized()

    qapp.exec()


if __name__ == "__main__":
    main()
```
在该目录下打开CMD（按住Shift->点击鼠标右键->在此处打开命令窗口/PowerShell）后运行下列命令启动VnpyRS：
```
python gui.py
```


## vnpy插件模式（将极速K线图表嵌入vnpy为其提供服务）

首先在vnpy的环境（或虚拟环境）中安装VnpyRS，例如通过pip安装：
```
pip install vnpyrs
```
编辑vnpy的源文件（如果是通过pip安装的话，它们就在安装目录或虚拟环境的Lib\site-packages里）vnpy_ctabacktester\ui\widget.py，在其顶部添加一行：
```Python
from vnpyrs import CandleChartDialog
```
找到文件中原有的类`CandleChartDialog`：
```Python
class CandleChartDialog(QtWidgets.QDialog):
```
将其改名，如改为`CandleChartDialog2`。
保存修改并重新运行vnpy


## 更新日志

0.1.1：支持sqlite和mysql数据库(2025-1-10)

0.1.2：修复BarData和TickData中的datetime与vnpy里的有差异的问题(2025-1-12)

0.2.0：vnpy自带的所有策略均通过了测试；支持GUI模式；支持K线图表及K线图表嵌入vnpy的模式；支持不带身份校验的MongoDB；修复若干重要bug