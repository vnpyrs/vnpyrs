# vnpyrs —— 与vnpy兼容但更快的回测框架

vnpyrs是以提升性能为目的，部分代码用Rust语言重新实现的vnpy。已实现回测和参数调优。

## 背景

众所周知，Python生态强大，编码灵活，但是有个缺点，就是慢。2024年，我实验性的用Rust重写了vnpy的回测模块，惊喜的发现，运行速度提升了近20倍。
显然，移植到Rust这件事的价值是巨大的。但还有个问题，由于它100%是用Rust编写的，包括策略也是用Rust编写的，那用户就无法使用Python的各种库了。况且，Rust是出了名的难入门的语言。上述的两点会降低它的实用性。

因此，在2025年我重新设定了目标，该项目必须完全兼容Python生态，100%兼容已经为vnpy编写的策略，同时要提升性能。
由于用户代码是用Python编写的，那么性能提升就不会像第一次用纯Rust编写时那样夸张了，但省下的时间依然不少。如果用examples文件夹里面的案例测试，在不改一行策略代码的情况下，在10核32G的电脑上，配合Python3.11，综合速度提升了一倍。（具体提升多少涉及硬件配置、Python版本和具体策略）

## 环境准备

vnpyrs对python包的依赖和vnpy几乎一样，但去掉了UI相关的包。Python版本需要3.7以上，推荐3.10以上

## pip安装

最佳的安装方式是源码编译安装，这样可以让pyo3启用针对特定的Python版本进行性能优化，而且大部分的深度vnpy用户都对代码做过修改，vnpyrs的代码基本上和vnpy原版的代码一一对应，这样您可以把对vnpy的修改移植到vnpyrs上去（虽然需要学习Rust，但是在有本工程大量范例的情况下不会很难）。

如果只是想试用一下，可以通过pip安装，但PYPI服务器上的版本一般是通用版本，未针对特定Python版本做过性能优化。

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

大体上和Linux一样，只是ta-lib Python版的安装可以依靠现成的whl包，可以从这里下载：https://github.com/cgohlke/talib-build/releases

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

vnpyrs是一个Rust和Python的混合项目，因此还需要安装maturin插件，您可以全局安装
```
pip install maturin
```
或者在创建Python虚拟环境后安装，创建虚拟环境的问题后面单独讲，Python社区更加建议用此方式。而且如果想修改vnpyrs代码的话，使用虚拟环境会更加容易

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

6.安装vnpyrs的whl文件

whl文件可以通过pip来安装。注意安装过程中会下载它所依赖的Python包，建议使用pip代理以加快下载速度，设置方法自行搜索
```
pip install (whl文件的文件名)
```
至此安装完成

7.（可选）建立Python虚拟环境，并以调试模式编译、运行vnpyrs

在项目根目录下执行以下命令
```
python3 -m venv .env
```
此时会多出来一个.env文件夹。执行以下命令进入该虚拟环境
```
source .env/bin/activate
```
之后再运行pip、python3、maturin命令的话，只影响该环境，或被该环境影响。直到退出shell会话

这个时候您无需运行“python3 -m maturin build -r”、“pip install (whl文件的文件名)”两条命令编译vnpyrs，而只需要一条：
```
python3 -m maturin develop -r
```
这个技巧在需要修改vnpyrs代码的时候特别有用


## 脚本运行

vnpyrs仅支持脚本运行，在任意目录下创建run.py，写入以下示例代码：

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

*vnpyrs使用的数据库和json配置文件和vnpy完全一样，二者是共用数据库的。标的300.LOCAL的数据在examples下，导入300_1min_vnpy.csv到vnpy引擎里即可。
*在该目录下打开CMD（按住Shift->点击鼠标右键->在此处打开命令窗口/PowerShell）后运行下列命令启动vnpyrs：
    python run.py


## 开发路线图

*不会完全移植vnpy。一个原因是vnpy用到了很多动态语言的特性，以实现插件化，这部分在静态语言上实现很困难，或者需要大量的unsafe。另一个原因是没必要，UI和网络模块即使通过Rust、C++重写，体验也不会明显变好，K线渲染可能例外，但那个更适合用OpenGL或Vulkan实现。

*未来可能会支持用Rust、C、C++写策略，这样的话回测性能提升10倍也是有可能的。

## 更新日志

0.1.1：支持sqlite和mysql数据库(2025-1-10)

0.1.2：修复BarData和TickData中的datetime与vnpy里的有差异的问题(2025-1-12)