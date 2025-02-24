from vnpyrs.backtesting import BacktestingEngine, BacktestingMode
from vnpyrs.trader.optimize import OptimizationSetting

# double_ma_strategy模块是vnpy自带的，复制到当前目录即可。对策略的导入必须在导入vnpyrs之后，除非将其中的vnpy_ctastrategy替换为vnpyrs
from double_ma_strategy import DoubleMaStrategy

from datetime import datetime
import time


def main():
    engine = BacktestingEngine()

    engine.set_parameters(
        vt_symbol="300.LOCAL",
        interval="1m",
        start=datetime(2009, 1, 5),
        end=datetime(2020, 8, 18),
        rate=2.3e-5,
        slippage=0.2,
        size=1,
        pricetick=0.2,
        capital=10000000,
    )

    start = time.perf_counter()
    engine.add_strategy(DoubleMaStrategy, {"fast_window": 10, "slow_window": 20})
    engine.load_data()
    engine.run_backtesting()
    df = engine.calculate_result()
    print(df)
    engine.calculate_statistics()
    engine.show_chart().show()

    setting = OptimizationSetting()
    setting.set_target("sharpe_ratio")
    setting.add_parameter("fast_window", 8, 12, 1)
    setting.add_parameter("slow_window", 12, 20, 1)
    engine.run_bf_optimization(setting)
    engine.run_ga_optimization(setting)
    print(
        time.perf_counter() - start
    )  # 这里用于统计耗时，可以看到vnpyrs耗时只有vnpy的一半


if __name__ == "__main__":
    main()
