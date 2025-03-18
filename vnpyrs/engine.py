from ast import List
import importlib
import traceback
from datetime import datetime
from threading import Thread
from pathlib import Path
from inspect import getfile
from glob import glob
from types import ModuleType
from pandas import DataFrame
from typing import Optional

from vnpyrs.optimize import OptimizationSetting

from .event_engine import Event, EventEngine

# from vnpy.trader.engine import BaseEngine, MainEngine
# from vnpy.trader.constant import Interval
# from vnpy.trader.utility import extract_vt_symbol
# from vnpy.trader.object import HistoryRequest, TickData, ContractData, BarData
# from vnpy.trader.database import BaseDatabase, get_database

from vnpyrs.backtesting import (
    BacktestingEngine,
)


def _(str):
    return str


APP_NAME = "CtaBacktester"

EVENT_BACKTESTER_LOG = "eBacktesterLog"
EVENT_BACKTESTER_BACKTESTING_FINISHED = "eBacktesterBacktestingFinished"
EVENT_BACKTESTER_OPTIMIZATION_FINISHED = "eBacktesterOptimizationFinished"


class BacktesterEngine:
    """
    For running CTA strategy backtesting.
    """

    def __init__(self) -> None:
        """"""
        self.event_engine = EventEngine()
        self.event_engine.start()

        self.classes: dict = {}
        self.backtesting_engine: BacktestingEngine = None
        self.thread: Thread = None

        # Backtesting reuslt
        self.result_df: DataFrame = None
        self.result_statistics: dict = None

        # Optimization result
        self.result_values: list = None

    def init_engine(self) -> None:
        """"""

        self.backtesting_engine = BacktestingEngine()
        # Redirect log from backtesting engine outside.
        self.backtesting_engine.set_output(self.write_log)

        self.load_strategy_class()
        self.write_log(_("策略文件加载完成"))

    def write_log(self, msg: str) -> None:
        """"""
        event: Event = Event(EVENT_BACKTESTER_LOG)
        event.data = msg
        self.event_engine.put(event)

    def load_strategy_class(self) -> None:
        """
        Load strategy class from source code.
        """
        path2: Path = Path.cwd().joinpath("strategies")
        self.load_strategy_class_from_folder(path2, "strategies")

    def load_strategy_class_from_folder(
        self, path: Path, module_name: str = ""
    ) -> None:
        """
        Load strategy class from certain folder.
        """
        for suffix in ["py", "pyd", "so"]:
            pathname: str = str(path.joinpath(f"*.{suffix}"))
            for filepath in glob(pathname):
                filename: str = Path(filepath).stem
                name: str = f"{module_name}.{filename}"
                self.load_strategy_class_from_module(name)

    def load_strategy_class_from_module(self, module_name: str) -> None:
        """
        Load strategy class from module file.
        """
        try:
            module: ModuleType = importlib.import_module(module_name)

            # 重载模块，确保如果策略文件中有任何修改，能够立即生效。
            importlib.reload(module)

            for name in dir(module):
                value = getattr(module, name)
                from vnpyrs import CtaTemplate, TargetPosTemplate

                if (
                    isinstance(value, type)
                    and issubclass(value, CtaTemplate)
                    and value not in {CtaTemplate, TargetPosTemplate}
                ):
                    self.classes[value.__name__] = value
        except:  # noqa
            msg: str = _("策略文件{}加载失败，触发异常：\n{}").format(
                module_name, traceback.format_exc()
            )
            self.write_log(msg)

    def reload_strategy_class(self) -> None:
        """"""
        self.classes.clear()
        self.load_strategy_class()
        self.write_log(_("策略文件重载刷新完成"))

    def get_strategy_class_names(self) -> list:
        """"""
        return list(self.classes.keys())

    def run_backtesting(
        self,
        class_name: str,
        vt_symbol: str,
        interval: str,
        start: datetime,
        end: datetime,
        rate: float,
        slippage: float,
        size: int,
        pricetick: float,
        capital: int,
        setting: dict,
    ) -> None:
        """"""
        self.result_df = None
        self.result_statistics = None

        engine: BacktestingEngine = self.backtesting_engine
        engine.clear_data()

        if interval == "tick":
            mode: str = "TICK"
        else:
            mode: str = "BAR"

        engine.set_parameters(
            vt_symbol=vt_symbol,
            interval=interval,
            start=start,
            end=end,
            rate=rate,
            slippage=slippage,
            size=size,
            pricetick=pricetick,
            capital=capital,
            mode=mode,
        )

        strategy_class: type = self.classes[class_name]
        engine.add_strategy(strategy_class, setting)

        engine.load_data()
        if not engine.has_history_data():
            self.write_log(_("策略回测失败，历史数据为空"))
            self.thread = None
            return

        try:
            engine.run_backtesting()
        except Exception:
            msg: str = _("策略回测失败，触发异常：\n{}").format(traceback.format_exc())
            self.write_log(msg)

            self.thread = None
            return

        self.result_df = engine.calculate_result()
        self.result_statistics = engine.calculate_statistics(output=False)

        # Clear thread object handler.
        self.thread = None

        # Put backtesting done event
        event: Event = Event(EVENT_BACKTESTER_BACKTESTING_FINISHED)
        self.event_engine.put(event)

    def start_backtesting(
        self,
        class_name: str,
        vt_symbol: str,
        interval: str,
        start: datetime,
        end: datetime,
        rate: float,
        slippage: float,
        size: int,
        pricetick: float,
        capital: int,
        setting: dict,
    ) -> bool:
        if self.thread:
            self.write_log(_("已有任务在运行中，请等待完成"))
            return False

        self.write_log("-" * 40)
        self.thread = Thread(
            target=self.run_backtesting,
            args=(
                class_name,
                vt_symbol,
                interval,
                start,
                end,
                rate,
                slippage,
                size,
                pricetick,
                capital,
                setting,
            ),
        )
        self.thread.start()

        return True

    def get_result_df(self) -> DataFrame:
        """"""
        return self.result_df

    def get_result_statistics(self) -> dict:
        """"""
        return self.result_statistics

    def get_result_values(self) -> list:
        """"""
        return self.result_values

    def get_default_setting(self, class_name: str) -> dict:
        """"""
        strategy_class: type = self.classes[class_name]
        return strategy_class.get_class_parameters()

    def run_optimization(
        self,
        class_name: str,
        vt_symbol: str,
        interval: str,
        start: datetime,
        end: datetime,
        rate: float,
        slippage: float,
        size: int,
        pricetick: float,
        capital: int,
        optimization_setting: OptimizationSetting,
        use_ga: bool,
        max_workers: int,
    ) -> None:
        """"""
        self.result_values = None

        engine: BacktestingEngine = self.backtesting_engine
        engine.clear_data()

        if interval == "tick":
            mode: str = "TICK"
        else:
            mode: str = "BAR"

        engine.set_parameters(
            vt_symbol=vt_symbol,
            interval=interval,
            start=start,
            end=end,
            rate=rate,
            slippage=slippage,
            size=size,
            pricetick=pricetick,
            capital=capital,
            mode=mode,
        )

        strategy_class: type = self.classes[class_name]
        engine.add_strategy(strategy_class, {})

        # 0则代表不限制
        if max_workers == 0:
            max_workers = None

        if use_ga:
            self.result_values = engine.run_ga_optimization(
                optimization_setting, output=False, max_workers=max_workers
            )
        else:
            self.result_values = engine.run_bf_optimization(
                optimization_setting, output=False, max_workers=max_workers
            )

        # Clear thread object handler.
        self.thread = None
        self.write_log(_("多进程参数优化完成"))

        # Put optimization done event
        event: Event = Event(EVENT_BACKTESTER_OPTIMIZATION_FINISHED)
        self.event_engine.put(event)

    def start_optimization(
        self,
        class_name: str,
        vt_symbol: str,
        interval: str,
        start: datetime,
        end: datetime,
        rate: float,
        slippage: float,
        size: int,
        pricetick: float,
        capital: int,
        optimization_setting: OptimizationSetting,
        use_ga: bool,
        max_workers: int,
    ) -> bool:
        if self.thread:
            self.write_log(_("已有任务在运行中，请等待完成"))
            return False

        self.write_log("-" * 40)
        self.thread = Thread(
            target=self.run_optimization,
            args=(
                class_name,
                vt_symbol,
                interval,
                start,
                end,
                rate,
                slippage,
                size,
                pricetick,
                capital,
                optimization_setting,
                use_ga,
                max_workers,
            ),
        )
        self.thread.start()

        return True

    def get_all_trades(self) -> list:
        """"""
        return self.backtesting_engine.get_all_trades()

    def get_all_orders(self) -> list:
        """"""
        return self.backtesting_engine.get_all_orders()

    def get_all_daily_results(self) -> list:
        """"""
        return self.backtesting_engine.get_all_daily_results()

    def get_history_data(self) -> list:
        """"""
        return self.backtesting_engine.history_data

    def get_strategy_class_file(self, class_name: str) -> str:
        """"""
        strategy_class: type = self.classes[class_name]
        file_path: str = getfile(strategy_class)
        return file_path
