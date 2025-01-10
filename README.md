# vnpyrs —— 与vnpy完全兼容但更快的回测框架

vnpyrs是以提升性能为目的，部分代码用Rust语言重新实现的vnpy。已实现回测和参数调优。

## 背景

众所周知，Python生态强大，编码灵活，但是有个缺点，就是慢。2024年，我实验性的用Rust重写了vnpy的回测模块，惊喜的发现，运行速度提升了近20倍。
显然，移植到Rust这件事的价值是巨大的。但还有个问题，由于它100%是用Rust编写的，包括策略也是用Rust编写的，那用户就无法使用Python的各种库了。况且，Rust是出了名的难入门的语言。上述的两点会降低它的实用性。
因此，在2025年我重新设定了目标，该项目必须完全兼容Python生态，100%兼容已经为vnpy编写的策略，同时要提升性能。
由于用户代码是用Python编写的，那性能提升就不会像第一次用纯Rust编写时那样夸张了，但省下的时间依然不少。如果用examples文件夹里面的案例测试，在不改一行策略代码的情况下，综合速度提升了一倍。（具体提升多少和策略的具体实现紧密相关）

## 环境准备

vnpyrs对python包的依赖和vnpy几乎一样，但去掉了UI相关的包。Python版本需要3.7以上，推荐3.10以上

## 安装步骤

**Windows**

```
pip install vnpyrs
```

**Linux**

```
pip install vnpyrs
```

**Macos**

需要git clone并源码编译


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
    print(time.perf_counter()-start) #这里用于统计耗时，可以看到vnpyrs耗时只有vnpy的一半

if __name__ == '__main__':
    main()
```

vnpyrs使用的数据库和json配置文件和vnpy完全一样，二者是共用数据库的。标的300.LOCAL的数据在examples下，导入300_1min_vnpy.csv到vnpy即可。
在该目录下打开CMD（按住Shift->点击鼠标右键->在此处打开命令窗口/PowerShell）后运行下列命令启动vnpyrs：
    python run.py


## 开发路线图

不会完全移植vnpy。
一个原因是vnpy用到了很多动态语言的特性，以实现插件化，这部分在静态语言上实现很困难，或者需要大量的unsafe；
另一个原因是没必要，UI和网络模块即使用Rust、C++重写，体验也不会明显变好。
但是未来会支持用Rust、C、C++写策略，这样的话回测性能提升10倍也是有可能的。

## 更新日志
0.1.1：支持sqlite和mysql数据库