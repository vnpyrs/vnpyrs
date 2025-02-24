from vnpyrs.widget import create_qapp, BacktesterWindow


def main():
    """"""
    qapp = create_qapp()
    backtester_window = BacktesterWindow()
    backtester_window.showMaximized()

    qapp.exec()


if __name__ == "__main__":
    main()
