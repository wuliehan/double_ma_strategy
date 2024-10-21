from vnpy_ctastrategy.backtesting import BacktestingEngine, BacktestingMode
from vnpy_ctastrategy.strategies.double_ma_strategy import DoubleMaStrategy
from vnpy.trader.optimize import OptimizationSetting

from datetime import datetime
import time

def main():
    engine=BacktestingEngine()

    engine.set_parameters(vt_symbol="ETH.LOCAL",
                        interval="1m",
                        start=datetime(2020,1,22),
                        end=datetime(2020,12,31),
                        rate=2.5e-5,
                        slippage=0.05,
                        size=1,
                        pricetick=0.01,
                        capital=1000000)

    # engine.set_parameters(vt_symbol="LTC-USDT.OKX",
    #                       interval="1m",
    #                       mode=BacktestingMode.TICK,
    #                       start=datetime(1999,12,24),
    #                       end=datetime(2024,6,4),
    #                       rate=2.5e-5,
    #                       slippage=0.2,
    #                       size=300,
    #                       pricetick=0.2,
    #                       capital=1000000)

    engine.add_strategy(DoubleMaStrategy,{'fast_window': 10, 'slow_window': 20})
    engine.load_data()
    start = time.perf_counter()
    engine.run_backtesting()
    print(time.perf_counter()-start)
    df=engine.calculate_result()
    engine.calculate_statistics()
    engine.show_chart()
    exit()

    setting=OptimizationSetting()
    setting.set_target("sharpe_ratio")
    setting.add_parameter("fast_window",8,12,1)
    setting.add_parameter("slow_window",12,20,1)
    # engine.run_optimization(setting)
    engine.run_bf_optimization(setting)


if __name__ == '__main__':
    main()