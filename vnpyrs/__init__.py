# import the contents of the Rust library into the Python extension
from vnpyrs.utility import load_json, save_json
from .vnpyrs import *
from .vnpyrs import __all__

# optional: include the documentation from the Rust module
from .vnpyrs import __doc__  # noqa: F401


from datetime import datetime, time
from typing import Callable, Dict, Tuple, Union, Optional
from functools import partial

import numpy as np
import talib
import subprocess

from abc import ABC
from copy import copy
from typing import Any, Callable, List

from vnpyrs.trader.constant import Interval, Direction, Offset_
from vnpyrs.trader.object import BarData, TickData, OrderData, TradeData
from vnpyrs.trader.utility import BarGenerator

import vnpyrs.optimize
import sys

sys.modules["vnpyrs.trader.optimize"] = vnpyrs.optimize
from vnpyrs.trader.optimize import (
    OptimizationSetting,
    check_optimization_setting,
    run_bf_optimization,
    run_ga_optimization,
)

from datetime import date, datetime, timedelta
from pandas import DataFrame
import plotly.graph_objects as go
from plotly.subplots import make_subplots
from vnpyrs.backtesting import BacktestingEngine, BacktestingMode
from vnpyrs import CandleChartDialog

vnpyrs.backtesting.OptimizationSetting = OptimizationSetting


def _(str):
    return str


class ArrayManager(object):
    """
    For:
    1. time series container of bar data
    2. calculating technical indicator value
    """

    def __init__(self, size: int = 100) -> None:
        """Constructor"""
        self.count: int = 0
        self.size: int = size
        self.inited: bool = False

        self.open_array: np.ndarray = np.zeros(size)
        self.high_array: np.ndarray = np.zeros(size)
        self.low_array: np.ndarray = np.zeros(size)
        self.close_array: np.ndarray = np.zeros(size)
        self.volume_array: np.ndarray = np.zeros(size)
        self.turnover_array: np.ndarray = np.zeros(size)
        self.open_interest_array: np.ndarray = np.zeros(size)

    def update_bar(self, bar: BarData) -> None:
        """
        Update new bar data into array manager.
        """
        self.count += 1
        if not self.inited and self.count >= self.size:
            self.inited = True

        self.open_array[:-1] = self.open_array[1:]
        self.high_array[:-1] = self.high_array[1:]
        self.low_array[:-1] = self.low_array[1:]
        self.close_array[:-1] = self.close_array[1:]
        self.volume_array[:-1] = self.volume_array[1:]
        self.turnover_array[:-1] = self.turnover_array[1:]
        self.open_interest_array[:-1] = self.open_interest_array[1:]

        self.open_array[-1] = bar.open_price
        self.high_array[-1] = bar.high_price
        self.low_array[-1] = bar.low_price
        self.close_array[-1] = bar.close_price
        self.volume_array[-1] = bar.volume
        self.turnover_array[-1] = bar.turnover
        self.open_interest_array[-1] = bar.open_interest

    @property
    def open(self) -> np.ndarray:
        """
        Get open price time series.
        """
        return self.open_array

    @property
    def high(self) -> np.ndarray:
        """
        Get high price time series.
        """
        return self.high_array

    @property
    def low(self) -> np.ndarray:
        """
        Get low price time series.
        """
        return self.low_array

    @property
    def close(self) -> np.ndarray:
        """
        Get close price time series.
        """
        return self.close_array

    @property
    def volume(self) -> np.ndarray:
        """
        Get trading volume time series.
        """
        return self.volume_array

    @property
    def turnover(self) -> np.ndarray:
        """
        Get trading turnover time series.
        """
        return self.turnover_array

    @property
    def open_interest(self) -> np.ndarray:
        """
        Get trading volume time series.
        """
        return self.open_interest_array

    def sma(self, n: int, array: bool = False) -> Union[float, np.ndarray]:
        """
        Simple moving average.
        """
        result: np.ndarray = talib.SMA(self.close, n)
        if array:
            return result
        return result[-1]

    def ema(self, n: int, array: bool = False) -> Union[float, np.ndarray]:
        """
        Exponential moving average.
        """
        result: np.ndarray = talib.EMA(self.close, n)
        if array:
            return result
        return result[-1]

    def kama(self, n: int, array: bool = False) -> Union[float, np.ndarray]:
        """
        KAMA.
        """
        result: np.ndarray = talib.KAMA(self.close, n)
        if array:
            return result
        return result[-1]

    def wma(self, n: int, array: bool = False) -> Union[float, np.ndarray]:
        """
        WMA.
        """
        result: np.ndarray = talib.WMA(self.close, n)
        if array:
            return result
        return result[-1]

    def apo(
        self, fast_period: int, slow_period: int, matype: int = 0, array: bool = False
    ) -> Union[float, np.ndarray]:
        """
        APO.
        """
        result: np.ndarray = talib.APO(self.close, fast_period, slow_period, matype)
        if array:
            return result
        return result[-1]

    def cmo(self, n: int, array: bool = False) -> Union[float, np.ndarray]:
        """
        CMO.
        """
        result: np.ndarray = talib.CMO(self.close, n)
        if array:
            return result
        return result[-1]

    def mom(self, n: int, array: bool = False) -> Union[float, np.ndarray]:
        """
        MOM.
        """
        result: np.ndarray = talib.MOM(self.close, n)
        if array:
            return result
        return result[-1]

    def ppo(
        self, fast_period: int, slow_period: int, matype: int = 0, array: bool = False
    ) -> Union[float, np.ndarray]:
        """
        PPO.
        """
        result: np.ndarray = talib.PPO(self.close, fast_period, slow_period, matype)
        if array:
            return result
        return result[-1]

    def roc(self, n: int, array: bool = False) -> Union[float, np.ndarray]:
        """
        ROC.
        """
        result: np.ndarray = talib.ROC(self.close, n)
        if array:
            return result
        return result[-1]

    def rocr(self, n: int, array: bool = False) -> Union[float, np.ndarray]:
        """
        ROCR.
        """
        result: np.ndarray = talib.ROCR(self.close, n)
        if array:
            return result
        return result[-1]

    def rocp(self, n: int, array: bool = False) -> Union[float, np.ndarray]:
        """
        ROCP.
        """
        result: np.ndarray = talib.ROCP(self.close, n)
        if array:
            return result
        return result[-1]

    def rocr_100(self, n: int, array: bool = False) -> Union[float, np.ndarray]:
        """
        ROCR100.
        """
        result: np.ndarray = talib.ROCR100(self.close, n)
        if array:
            return result
        return result[-1]

    def trix(self, n: int, array: bool = False) -> Union[float, np.ndarray]:
        """
        TRIX.
        """
        result: np.ndarray = talib.TRIX(self.close, n)
        if array:
            return result
        return result[-1]

    def std(
        self, n: int, nbdev: int = 1, array: bool = False
    ) -> Union[float, np.ndarray]:
        """
        Standard deviation.
        """
        result: np.ndarray = talib.STDDEV(self.close, n, nbdev)
        if array:
            return result
        return result[-1]

    def obv(self, array: bool = False) -> Union[float, np.ndarray]:
        """
        OBV.
        """
        result: np.ndarray = talib.OBV(self.close, self.volume)
        if array:
            return result
        return result[-1]

    def cci(self, n: int, array: bool = False) -> Union[float, np.ndarray]:
        """
        Commodity Channel Index (CCI).
        """
        result: np.ndarray = talib.CCI(self.high, self.low, self.close, n)
        if array:
            return result
        return result[-1]

    def atr(self, n: int, array: bool = False) -> Union[float, np.ndarray]:
        """
        Average True Range (ATR).
        """
        result: np.ndarray = talib.ATR(self.high, self.low, self.close, n)
        if array:
            return result
        return result[-1]

    def natr(self, n: int, array: bool = False) -> Union[float, np.ndarray]:
        """
        NATR.
        """
        result: np.ndarray = talib.NATR(self.high, self.low, self.close, n)
        if array:
            return result
        return result[-1]

    def rsi(self, n: int, array: bool = False) -> Union[float, np.ndarray]:
        """
        Relative Strenght Index (RSI).
        """
        result: np.ndarray = talib.RSI(self.close, n)
        if array:
            return result
        return result[-1]

    def macd(
        self,
        fast_period: int,
        slow_period: int,
        signal_period: int,
        array: bool = False,
    ) -> Union[Tuple[np.ndarray, np.ndarray, np.ndarray], Tuple[float, float, float]]:
        """
        MACD.
        """
        macd, signal, hist = talib.MACD(
            self.close, fast_period, slow_period, signal_period
        )
        if array:
            return macd, signal, hist
        return macd[-1], signal[-1], hist[-1]

    def adx(self, n: int, array: bool = False) -> Union[float, np.ndarray]:
        """
        ADX.
        """
        result: np.ndarray = talib.ADX(self.high, self.low, self.close, n)
        if array:
            return result
        return result[-1]

    def adxr(self, n: int, array: bool = False) -> Union[float, np.ndarray]:
        """
        ADXR.
        """
        result: np.ndarray = talib.ADXR(self.high, self.low, self.close, n)
        if array:
            return result
        return result[-1]

    def dx(self, n: int, array: bool = False) -> Union[float, np.ndarray]:
        """
        DX.
        """
        result: np.ndarray = talib.DX(self.high, self.low, self.close, n)
        if array:
            return result
        return result[-1]

    def minus_di(self, n: int, array: bool = False) -> Union[float, np.ndarray]:
        """
        MINUS_DI.
        """
        result: np.ndarray = talib.MINUS_DI(self.high, self.low, self.close, n)
        if array:
            return result
        return result[-1]

    def plus_di(self, n: int, array: bool = False) -> Union[float, np.ndarray]:
        """
        PLUS_DI.
        """
        result: np.ndarray = talib.PLUS_DI(self.high, self.low, self.close, n)
        if array:
            return result
        return result[-1]

    def willr(self, n: int, array: bool = False) -> Union[float, np.ndarray]:
        """
        WILLR.
        """
        result: np.ndarray = talib.WILLR(self.high, self.low, self.close, n)
        if array:
            return result
        return result[-1]

    def ultosc(
        self,
        time_period1: int = 7,
        time_period2: int = 14,
        time_period3: int = 28,
        array: bool = False,
    ) -> Union[float, np.ndarray]:
        """
        Ultimate Oscillator.
        """
        result: np.ndarray = talib.ULTOSC(
            self.high, self.low, self.close, time_period1, time_period2, time_period3
        )
        if array:
            return result
        return result[-1]

    def trange(self, array: bool = False) -> Union[float, np.ndarray]:
        """
        TRANGE.
        """
        result: np.ndarray = talib.TRANGE(self.high, self.low, self.close)
        if array:
            return result
        return result[-1]

    def boll(
        self, n: int, dev: float, array: bool = False
    ) -> Union[Tuple[np.ndarray, np.ndarray], Tuple[float, float]]:
        """
        Bollinger Channel.
        """
        mid: Union[float, np.ndarray] = self.sma(n, array)
        std: Union[float, np.ndarray] = self.std(n, 1, array)

        up: Union[float, np.ndarray] = mid + std * dev
        down: Union[float, np.ndarray] = mid - std * dev

        return up, down

    def keltner(
        self, n: int, dev: float, array: bool = False
    ) -> Union[Tuple[np.ndarray, np.ndarray], Tuple[float, float]]:
        """
        Keltner Channel.
        """
        mid: Union[float, np.ndarray] = self.sma(n, array)
        atr: Union[float, np.ndarray] = self.atr(n, array)

        up: Union[float, np.ndarray] = mid + atr * dev
        down: Union[float, np.ndarray] = mid - atr * dev

        return up, down

    def donchian(
        self, n: int, array: bool = False
    ) -> Union[Tuple[np.ndarray, np.ndarray], Tuple[float, float]]:
        """
        Donchian Channel.
        """
        up: np.ndarray = talib.MAX(self.high, n)
        down: np.ndarray = talib.MIN(self.low, n)

        if array:
            return up, down
        return up[-1], down[-1]

    def aroon(
        self, n: int, array: bool = False
    ) -> Union[Tuple[np.ndarray, np.ndarray], Tuple[float, float]]:
        """
        Aroon indicator.
        """
        aroon_down, aroon_up = talib.AROON(self.high, self.low, n)

        if array:
            return aroon_up, aroon_down
        return aroon_up[-1], aroon_down[-1]

    def aroonosc(self, n: int, array: bool = False) -> Union[float, np.ndarray]:
        """
        Aroon Oscillator.
        """
        result: np.ndarray = talib.AROONOSC(self.high, self.low, n)

        if array:
            return result
        return result[-1]

    def minus_dm(self, n: int, array: bool = False) -> Union[float, np.ndarray]:
        """
        MINUS_DM.
        """
        result: np.ndarray = talib.MINUS_DM(self.high, self.low, n)

        if array:
            return result
        return result[-1]

    def plus_dm(self, n: int, array: bool = False) -> Union[float, np.ndarray]:
        """
        PLUS_DM.
        """
        result: np.ndarray = talib.PLUS_DM(self.high, self.low, n)

        if array:
            return result
        return result[-1]

    def mfi(self, n: int, array: bool = False) -> Union[float, np.ndarray]:
        """
        Money Flow Index.
        """
        result: np.ndarray = talib.MFI(self.high, self.low, self.close, self.volume, n)
        if array:
            return result
        return result[-1]

    def ad(self, array: bool = False) -> Union[float, np.ndarray]:
        """
        AD.
        """
        result: np.ndarray = talib.AD(self.high, self.low, self.close, self.volume)
        if array:
            return result
        return result[-1]

    def adosc(
        self, fast_period: int, slow_period: int, array: bool = False
    ) -> Union[float, np.ndarray]:
        """
        ADOSC.
        """
        result: np.ndarray = talib.ADOSC(
            self.high, self.low, self.close, self.volume, fast_period, slow_period
        )
        if array:
            return result
        return result[-1]

    def bop(self, array: bool = False) -> Union[float, np.ndarray]:
        """
        BOP.
        """
        result: np.ndarray = talib.BOP(self.open, self.high, self.low, self.close)

        if array:
            return result
        return result[-1]

    def stoch(
        self,
        fastk_period: int,
        slowk_period: int,
        slowk_matype: int,
        slowd_period: int,
        slowd_matype: int,
        array: bool = False,
    ) -> Union[Tuple[float, float], Tuple[np.ndarray, np.ndarray]]:
        """
        Stochastic Indicator
        """
        k, d = talib.STOCH(
            self.high,
            self.low,
            self.close,
            fastk_period,
            slowk_period,
            slowk_matype,
            slowd_period,
            slowd_matype,
        )
        if array:
            return k, d
        return k[-1], d[-1]


def virtual(func: Callable) -> Callable:
    """
    mark a function as "virtual", which means that this function can be override.
    any base class should use this or @abstractmethod to decorate all functions
    that can be (re)implemented by subclasses.
    """
    return func


class CtaTemplate(ABC):
    """"""

    author: str = ""
    parameters: list = []
    variables: list = []

    def __init__(
        self,
        cta_engine: Any,
        strategy_name: str,
        vt_symbol: str,
        setting: dict,
    ) -> None:
        """"""
        self.cta_engine: Any = cta_engine
        self.strategy_name: str = strategy_name
        self.vt_symbol: str = vt_symbol

        self.inited: bool = False
        self.trading: bool = False
        self.pos: int = 0

        # Copy a new variables list here to avoid duplicate insert when multiple
        # strategy instances are created with the same strategy class.
        self.variables = copy(self.variables)
        self.variables.insert(0, "inited")
        self.variables.insert(1, "trading")
        self.variables.insert(2, "pos")

        self.update_setting(setting)

    def update_setting(self, setting: dict) -> None:
        """
        Update strategy parameter wtih value in setting dict.
        """
        for name in self.parameters:
            if name in setting:
                setattr(self, name, setting[name])

    @classmethod
    def get_class_parameters(cls) -> dict:
        """
        Get default parameters dict of strategy class.
        """
        class_parameters: dict = {}
        for name in cls.parameters:
            class_parameters[name] = getattr(cls, name)
        return class_parameters

    def get_parameters(self) -> dict:
        """
        Get strategy parameters dict.
        """
        strategy_parameters: dict = {}
        for name in self.parameters:
            strategy_parameters[name] = getattr(self, name)
        return strategy_parameters

    def get_variables(self) -> dict:
        """
        Get strategy variables dict.
        """
        strategy_variables: dict = {}
        for name in self.variables:
            strategy_variables[name] = getattr(self, name)
        return strategy_variables

    def get_data(self) -> dict:
        """
        Get strategy data.
        """
        strategy_data: dict = {
            "strategy_name": self.strategy_name,
            "vt_symbol": self.vt_symbol,
            "class_name": self.__class__.__name__,
            "author": self.author,
            "parameters": self.get_parameters(),
            "variables": self.get_variables(),
        }
        return strategy_data

    @virtual
    def on_init(self) -> None:
        """
        Callback when strategy is inited.
        """
        pass

    @virtual
    def on_start(self) -> None:
        """
        Callback when strategy is started.
        """
        pass

    @virtual
    def on_stop(self) -> None:
        """
        Callback when strategy is stopped.
        """
        pass

    @virtual
    def on_tick(self, tick: TickData) -> None:
        """
        Callback of new tick data update.
        """
        pass

    @virtual
    def on_bar(self, bar: BarData) -> None:
        """
        Callback of new bar data update.
        """
        pass

    @virtual
    def on_trade(self, trade: TradeData) -> None:
        """
        Callback of new trade data update.
        """
        pass

    @virtual
    def on_order(self, order: OrderData) -> None:
        """
        Callback of new order data update.
        """
        pass

    @virtual
    def on_stop_order(self, stop_order: StopOrder) -> None:
        """
        Callback of stop order update.
        """
        pass

    def buy(
        self,
        price: float,
        volume: float,
        stop: bool = False,
        lock: bool = False,
        net: bool = False,
    ) -> list:
        """
        Send buy order to open a long position.
        """
        return self.send_order(
            Direction.LONG, Offset_.OPEN, price, volume, stop, lock, net
        )

    def sell(
        self,
        price: float,
        volume: float,
        stop: bool = False,
        lock: bool = False,
        net: bool = False,
    ) -> list:
        """
        Send sell order to close a long position.
        """
        return self.send_order(
            Direction.SHORT, Offset_.CLOSE, price, volume, stop, lock, net
        )

    def short(
        self,
        price: float,
        volume: float,
        stop: bool = False,
        lock: bool = False,
        net: bool = False,
    ) -> list:
        """
        Send short order to open as short position.
        """
        return self.send_order(
            Direction.SHORT, Offset_.OPEN, price, volume, stop, lock, net
        )

    def cover(
        self,
        price: float,
        volume: float,
        stop: bool = False,
        lock: bool = False,
        net: bool = False,
    ) -> list:
        """
        Send cover order to close a short position.
        """
        return self.send_order(
            Direction.LONG, Offset_.CLOSE, price, volume, stop, lock, net
        )

    def send_order(
        self,
        direction: Direction,
        offset: Offset_,
        price: float,
        volume: float,
        stop: bool = False,
        lock: bool = False,
        net: bool = False,
    ) -> list:
        """
        Send a new order.
        """
        if self.trading:
            vt_orderids: list = self.cta_engine.send_order(
                self, direction, offset, price, volume, stop, lock, net
            )
            return vt_orderids
        else:
            return []

    def cancel_order(self, vt_orderid: str) -> None:
        """
        Cancel an existing order.
        """
        if self.trading:
            self.cta_engine.cancel_order(self, vt_orderid)

    def cancel_all(self) -> None:
        """
        Cancel all orders sent by strategy.
        """
        if self.trading:
            self.cta_engine.cancel_all(self)

    def write_log(self, msg: str) -> None:
        """
        Write a log message.
        """
        self.cta_engine.write_log(msg, self)

    def get_engine_type(self) -> EngineType:
        """
        Return whether the cta_engine is backtesting or live trading.
        """
        return self.cta_engine.get_engine_type()

    def get_pricetick(self) -> float:
        """
        Return pricetick data of trading contract.
        """
        return self.cta_engine.get_pricetick(self)

    def get_size(self) -> int:
        """
        Return size data of trading contract.
        """
        return self.cta_engine.get_size(self)

    def load_bar(
        self,
        days: int,
        interval: Interval = Interval.MINUTE,
        callback: Callable = None,
        use_database: bool = False,
    ) -> None:
        """
        Load historical bar data for initializing strategy.
        """
        if not callback:
            callback: Callable = self.on_bar

        bars: List[BarData] = self.cta_engine.load_bar(
            self.vt_symbol, days, interval, callback, use_database
        )

        for bar in bars:
            callback(bar)

    def load_tick(self, days: int) -> None:
        """
        Load historical tick data for initializing strategy.
        """
        ticks: List[TickData] = self.cta_engine.load_tick(
            self.vt_symbol, days, self.on_tick
        )

        for tick in ticks:
            self.on_tick(tick)

    def put_event(self) -> None:
        """
        Put an strategy data event for ui update.
        """
        if self.inited:
            self.cta_engine.put_strategy_event(self)

    def send_email(self, msg) -> None:
        """
        Send email to default receiver.
        """
        if self.inited:
            self.cta_engine.send_email(msg, self)

    def sync_data(self) -> None:
        """
        Sync strategy variables value into disk storage.
        """
        if self.trading:
            self.cta_engine.sync_strategy_data(self)


class CtaSignal(ABC):
    """"""

    def __init__(self) -> None:
        """"""
        self.signal_pos = 0

    @virtual
    def on_tick(self, tick: TickData) -> None:
        """
        Callback of new tick data update.
        """
        pass

    @virtual
    def on_bar(self, bar: BarData) -> None:
        """
        Callback of new bar data update.
        """
        pass

    def set_signal_pos(self, pos) -> None:
        """"""
        self.signal_pos = pos

    def get_signal_pos(self) -> Any:
        """"""
        return self.signal_pos


class TargetPosTemplate(CtaTemplate):
    """"""

    tick_add = 1

    last_tick: TickData = None
    last_bar: BarData = None
    target_pos = 0

    def __init__(self, cta_engine, strategy_name, vt_symbol, setting) -> None:
        """"""
        super().__init__(cta_engine, strategy_name, vt_symbol, setting)

        self.active_orderids: list = []
        self.cancel_orderids: list = []

        self.variables.append("target_pos")

    @virtual
    def on_tick(self, tick: TickData) -> None:
        """
        Callback of new tick data update.
        """
        self.last_tick = tick

    @virtual
    def on_bar(self, bar: BarData) -> None:
        """
        Callback of new bar data update.
        """
        self.last_bar = bar

    @virtual
    def on_order(self, order: OrderData) -> None:
        """
        Callback of new order data update.
        """
        vt_orderid: str = order.vt_orderid

        if not order.is_active():
            if vt_orderid in self.active_orderids:
                self.active_orderids.remove(vt_orderid)

            if vt_orderid in self.cancel_orderids:
                self.cancel_orderids.remove(vt_orderid)

    def check_order_finished(self) -> bool:
        """"""
        if self.active_orderids:
            return False
        else:
            return True

    def set_target_pos(self, target_pos) -> None:
        """"""
        self.target_pos = target_pos
        self.trade()

    def trade(self) -> None:
        """"""
        if not self.check_order_finished():
            self.cancel_old_order()
        else:
            self.send_new_order()

    def cancel_old_order(self) -> None:
        """"""
        for vt_orderid in self.active_orderids:
            if vt_orderid not in self.cancel_orderids:
                self.cancel_order(vt_orderid)
                self.cancel_orderids.append(vt_orderid)

    def send_new_order(self) -> None:
        """"""
        pos_change = self.target_pos - self.pos
        if not pos_change:
            return

        long_price = 0
        short_price = 0

        if self.last_tick:
            if pos_change > 0:
                long_price = self.last_tick.ask_price_1 + self.tick_add
                if self.last_tick.limit_up:
                    long_price = min(long_price, self.last_tick.limit_up)
            else:
                short_price = self.last_tick.bid_price_1 - self.tick_add
                if self.last_tick.limit_down:
                    short_price = max(short_price, self.last_tick.limit_down)

        else:
            if pos_change > 0:
                long_price = self.last_bar.close_price + self.tick_add
            else:
                short_price = self.last_bar.close_price - self.tick_add

        if self.get_engine_type() == EngineType.BACKTESTING:
            if pos_change > 0:
                vt_orderids: list = self.buy(long_price, abs(pos_change))
            else:
                vt_orderids: list = self.short(short_price, abs(pos_change))
            self.active_orderids.extend(vt_orderids)

        else:
            if self.active_orderids:
                return

            if pos_change > 0:
                if self.pos < 0:
                    if pos_change < abs(self.pos):
                        vt_orderids: list = self.cover(long_price, pos_change)
                    else:
                        vt_orderids: list = self.cover(long_price, abs(self.pos))
                else:
                    vt_orderids: list = self.buy(long_price, abs(pos_change))
            else:
                if self.pos > 0:
                    if abs(pos_change) < self.pos:
                        vt_orderids: list = self.sell(short_price, abs(pos_change))
                    else:
                        vt_orderids: list = self.sell(short_price, abs(self.pos))
                else:
                    vt_orderids: list = self.short(short_price, abs(pos_change))
            self.active_orderids.extend(vt_orderids)


def member_calculate_statistics(self, df: DataFrame = None, output=True) -> dict:
    """"""
    self.output(_("开始计算策略统计指标"))

    # Check DataFrame input exterior
    if df is None:
        df: DataFrame = self.daily_df

    # Init all statistics default value
    start_date: str = ""
    end_date: str = ""
    total_days: int = 0
    profit_days: int = 0
    loss_days: int = 0
    end_balance: float = 0
    max_drawdown: float = 0
    max_ddpercent: float = 0
    max_drawdown_duration: int = 0
    total_net_pnl: float = 0
    daily_net_pnl: float = 0
    total_commission: float = 0
    daily_commission: float = 0
    total_slippage: float = 0
    daily_slippage: float = 0
    total_turnover: float = 0
    daily_turnover: float = 0
    total_trade_count: int = 0
    daily_trade_count: float = 0
    total_return: float = 0
    annual_return: float = 0
    daily_return: float = 0
    return_std: float = 0
    sharpe_ratio: float = 0
    ewm_sharpe: float = 0
    return_drawdown_ratio: float = 0

    # Check if balance is always positive
    positive_balance: bool = False

    if df is not None:
        # Calculate balance related time series data
        df["balance"] = df["net_pnl"].cumsum() + self.capital

        # When balance falls below 0, set daily return to 0
        pre_balance: Series = df["balance"].shift(1)
        pre_balance.iloc[0] = self.capital
        x = df["balance"] / pre_balance
        x[x <= 0] = np.nan
        df["return"] = np.log(x).fillna(0)

        df["highlevel"] = (
            df["balance"].rolling(min_periods=1, window=len(df), center=False).max()
        )
        df["drawdown"] = df["balance"] - df["highlevel"]
        df["ddpercent"] = df["drawdown"] / df["highlevel"] * 100

        # All balance value needs to be positive
        positive_balance = (df["balance"] > 0).all()
        if not positive_balance:
            self.output(_("回测中出现爆仓（资金小于等于0），无法计算策略统计指标"))

    # Calculate statistics value
    if positive_balance:
        # Calculate statistics value
        start_date = df.index[0]
        end_date = df.index[-1]

        total_days: int = len(df)
        profit_days: int = len(df[df["net_pnl"] > 0])
        loss_days: int = len(df[df["net_pnl"] < 0])

        end_balance = df["balance"].iloc[-1]
        max_drawdown = df["drawdown"].min()
        max_ddpercent = df["ddpercent"].min()
        max_drawdown_end = df["drawdown"].idxmin()

        if isinstance(max_drawdown_end, date):
            max_drawdown_start = df["balance"][:max_drawdown_end].idxmax()
            max_drawdown_duration: int = (max_drawdown_end - max_drawdown_start).days
        else:
            max_drawdown_duration: int = 0

        total_net_pnl: float = df["net_pnl"].sum()
        daily_net_pnl: float = total_net_pnl / total_days

        total_commission: float = df["commission"].sum()
        daily_commission: float = total_commission / total_days

        total_slippage: float = df["slippage"].sum()
        daily_slippage: float = total_slippage / total_days

        total_turnover: float = df["turnover"].sum()
        daily_turnover: float = total_turnover / total_days

        total_trade_count: int = df["trade_count"].sum()
        daily_trade_count: int = total_trade_count / total_days

        total_return: float = (end_balance / self.capital - 1) * 100
        annual_return: float = total_return / total_days * self.annual_days
        daily_return: float = df["return"].mean() * 100
        return_std: float = df["return"].std() * 100

        if return_std:
            daily_risk_free: float = self.risk_free / np.sqrt(self.annual_days)
            sharpe_ratio: float = (
                (daily_return - daily_risk_free)
                / return_std
                * np.sqrt(self.annual_days)
            )

            ewm_window: ExponentialMovingWindow = df["return"].ewm(
                halflife=self.half_life
            )
            ewm_mean: Series = ewm_window.mean() * 100
            ewm_std: Series = ewm_window.std() * 100
            ewm_sharpe: float = ((ewm_mean - daily_risk_free) / ewm_std)[-1] * np.sqrt(
                self.annual_days
            )
        else:
            sharpe_ratio: float = 0
            ewm_sharpe: float = 0

        if max_ddpercent:
            return_drawdown_ratio: float = -total_return / max_ddpercent
        else:
            return_drawdown_ratio = 0

    # Output
    if output:
        self.output("-" * 30)
        self.output(_("首个交易日：\t{}").format(start_date))
        self.output(_("最后交易日：\t{}").format(end_date))

        self.output(_("总交易日：\t{}").format(total_days))
        self.output(_("盈利交易日：\t{}").format(profit_days))
        self.output(_("亏损交易日：\t{}").format(loss_days))

        self.output(_("起始资金：\t{:,.2f}").format(self.capital))
        self.output(_("结束资金：\t{:,.2f}").format(end_balance))

        self.output(_("总收益率：\t{:,.2f}%").format(total_return))
        self.output(_("年化收益：\t{:,.2f}%").format(annual_return))
        self.output(_("最大回撤: \t{:,.2f}").format(max_drawdown))
        self.output(_("百分比最大回撤: {:,.2f}%").format(max_ddpercent))
        self.output(_("最大回撤天数: \t{}").format(max_drawdown_duration))

        self.output(_("总盈亏：\t{:,.2f}").format(total_net_pnl))
        self.output(_("总手续费：\t{:,.2f}").format(total_commission))
        self.output(_("总滑点：\t{:,.2f}").format(total_slippage))
        self.output(_("总成交金额：\t{:,.2f}").format(total_turnover))
        self.output(_("总成交笔数：\t{}").format(total_trade_count))

        self.output(_("日均盈亏：\t{:,.2f}").format(daily_net_pnl))
        self.output(_("日均手续费：\t{:,.2f}").format(daily_commission))
        self.output(_("日均滑点：\t{:,.2f}").format(daily_slippage))
        self.output(_("日均成交金额：\t{:,.2f}").format(daily_turnover))
        self.output(_("日均成交笔数：\t{}").format(daily_trade_count))

        self.output(_("日均收益率：\t{:,.2f}%").format(daily_return))
        self.output(_("收益标准差：\t{:,.2f}%").format(return_std))
        self.output(f"Sharpe Ratio：\t{sharpe_ratio:,.2f}")
        self.output(f"EWM Sharpe：\t{ewm_sharpe:,.2f}")
        self.output(_("收益回撤比：\t{:,.2f}").format(return_drawdown_ratio))

    statistics: dict = {
        "start_date": start_date,
        "end_date": end_date,
        "total_days": total_days,
        "profit_days": profit_days,
        "loss_days": loss_days,
        "capital": self.capital,
        "end_balance": end_balance,
        "max_drawdown": max_drawdown,
        "max_ddpercent": max_ddpercent,
        "max_drawdown_duration": max_drawdown_duration,
        "total_net_pnl": total_net_pnl,
        "daily_net_pnl": daily_net_pnl,
        "total_commission": total_commission,
        "daily_commission": daily_commission,
        "total_slippage": total_slippage,
        "daily_slippage": daily_slippage,
        "total_turnover": total_turnover,
        "daily_turnover": daily_turnover,
        "total_trade_count": total_trade_count,
        "daily_trade_count": daily_trade_count,
        "total_return": total_return,
        "annual_return": annual_return,
        "daily_return": daily_return,
        "return_std": return_std,
        "sharpe_ratio": sharpe_ratio,
        "ewm_sharpe": ewm_sharpe,
        "return_drawdown_ratio": return_drawdown_ratio,
    }

    # Filter potential error infinite value
    for key, value in statistics.items():
        if value in (np.inf, -np.inf):
            value = 0
        statistics[key] = np.nan_to_num(value)

    self.output(_("策略统计指标计算完成"))
    return statistics


def member_show_chart(self, df: DataFrame = None) -> go.Figure:
    """"""
    # Check DataFrame input exterior
    if df is None:
        df: DataFrame = self.daily_df

    # Check for init DataFrame
    if df is None:
        return

    fig = make_subplots(
        rows=4,
        cols=1,
        subplot_titles=["Balance", "Drawdown", "Daily Pnl", "Pnl Distribution"],
        vertical_spacing=0.06,
    )

    balance_line = go.Scatter(x=df.index, y=df["balance"], mode="lines", name="Balance")

    drawdown_scatter = go.Scatter(
        x=df.index,
        y=df["drawdown"],
        fillcolor="red",
        fill="tozeroy",
        mode="lines",
        name="Drawdown",
    )
    pnl_bar = go.Bar(y=df["net_pnl"], name="Daily Pnl")
    pnl_histogram = go.Histogram(x=df["net_pnl"], nbinsx=100, name="Days")

    fig.add_trace(balance_line, row=1, col=1)
    fig.add_trace(drawdown_scatter, row=2, col=1)
    fig.add_trace(pnl_bar, row=3, col=1)
    fig.add_trace(pnl_histogram, row=4, col=1)

    fig.update_layout(height=1000, width=1000)
    return fig


def member_run_bf_optimization(
    self,
    optimization_setting: OptimizationSetting,
    output: bool = True,
    max_workers: int = None,
) -> list:
    """"""
    if not check_optimization_setting(optimization_setting):
        return

    evaluate_func: callable = wrap_evaluate(self, optimization_setting.target_name)
    results: list = run_bf_optimization(
        evaluate_func,
        optimization_setting,
        get_target_value,
        max_workers=max_workers,
        output=self.output,
    )

    if output:
        for result in results:
            msg: str = _("参数：{}, 目标：{}").format(result[0], result[1])
            self.output(msg)

    return results


def member_run_ga_optimization(
    self,
    optimization_setting: OptimizationSetting,
    output: bool = True,
    max_workers: int = None,
    ngen_size: int = 30,
) -> list:
    """"""
    if not check_optimization_setting(optimization_setting):
        return

    evaluate_func: callable = wrap_evaluate(self, optimization_setting.target_name)
    results: list = run_ga_optimization(
        evaluate_func,
        optimization_setting,
        get_target_value,
        max_workers=max_workers,
        ngen_size=ngen_size,
        output=self.output,
    )

    if output:
        for result in results:
            msg: str = _("参数：{}, 目标：{}").format(result[0], result[1])
            self.output(msg)

    return results


def evaluate(
    target_name: str,
    strategy_class: CtaTemplate,
    vt_symbol: str,
    interval: Interval,
    start: datetime,
    rate: float,
    slippage: float,
    size: float,
    pricetick: float,
    capital: int,
    end: datetime,
    mode: BacktestingMode,
    setting: dict,
) -> tuple:
    """
    Function for running in multiprocessing.pool
    """
    engine: BacktestingEngine = BacktestingEngine()
    engine.rs_use_global_data = True

    engine.set_parameters(
        vt_symbol=vt_symbol,
        interval=interval,
        start=start,
        rate=rate,
        slippage=slippage,
        size=size,
        pricetick=pricetick,
        capital=capital,
        end=end,
        mode=mode,
    )

    engine.add_strategy(strategy_class, setting)
    engine.load_data()
    engine.run_backtesting()
    engine.calculate_result()
    statistics: dict = engine.calculate_statistics(output=False)

    target_value: float = statistics[target_name]
    return (setting, target_value, statistics)


def wrap_evaluate(engine: BacktestingEngine, target_name: str) -> callable:
    """
    Wrap evaluate function with given setting from backtesting engine.
    """
    func: callable = partial(
        evaluate,
        target_name,
        engine.strategy_class,
        engine.vt_symbol,
        str(engine.interval),
        engine.start,
        engine.rate,
        engine.slippage,
        engine.size,
        engine.pricetick,
        engine.capital,
        engine.end,
        str(engine.mode),
    )
    return func


def get_target_value(result: list) -> float:
    """
    Get target value for sorting optimization results.
    """
    return result[1]


BacktestingEngine.calculate_statistics = member_calculate_statistics
BacktestingEngine.show_chart = member_show_chart
BacktestingEngine.run_bf_optimization = member_run_bf_optimization
BacktestingEngine.run_optimization = member_run_bf_optimization
BacktestingEngine.run_ga_optimization = member_run_ga_optimization


def member_exec_(self):
    from PySide6.QtWidgets import QFileDialog, QMessageBox

    setting: dict = load_json("vnpyrs.json")
    chart_path = setting.get("chartpath", "")
    if chart_path == "":
        QMessageBox.information(
            None,
            "注意",
            "这可能是您第一次运行vnpyrs的K线图表，请选择vnpyrs-chart(.exe)文件的路径，该文件可以从 https://github.com/vnpyrs/vnpyrs-chart/releases 下载或自己编译。程序会记录这次选择，以后可以在vnpy的配置目录中找到vnpyrs.json修改",
            QMessageBox.Ok,
        )
        file_path, _ = QFileDialog.getOpenFileName(
            caption="请选择vnpyrs-chart(.exe)文件的路径"
        )
        try:
            subprocess.run([file_path])
        except:
            raise
        else:
            setting["chartpath"] = file_path
            save_json("vnpyrs.json", setting)
    else:
        subprocess.run([chart_path])


CandleChartDialog.exec_ = member_exec_

if "vnpy" not in sys.modules.keys():
    sys.modules["vnpy_ctastrategy"] = vnpyrs
