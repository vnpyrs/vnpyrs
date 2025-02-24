import csv
from typing import Any, Dict
from PySide6 import QtCore, QtWidgets, QtGui

from vnpyrs.event_engine import Event, EventEngine

COLOR_LONG = QtGui.QColor("red")
COLOR_SHORT = QtGui.QColor("green")
COLOR_NET = QtGui.QColor("white")


def _(str):
    return str


class BaseCell(QtWidgets.QTableWidgetItem):
    """
    General cell used in tablewidgets.
    """

    def __init__(self, content, data) -> None:
        """"""
        super().__init__()
        self.setTextAlignment(QtCore.Qt.AlignmentFlag.AlignCenter)
        self.set_content(content, data)

    def set_content(self, content, data) -> None:
        """
        Set text content.
        """
        self.setText(str(content))
        self._data = data

    def get_data(self):
        """
        Get data object.
        """
        return self._data


class EnumCell(BaseCell):
    """
    Cell used for showing enum data.
    """

    def __init__(self, content: str, data) -> None:
        """"""
        super().__init__(content, data)

    def set_content(self, content, data) -> None:
        """
        Set text using enum.constant.value.
        """
        if content:
            super().set_content(content, data)


class DirectionCell(EnumCell):
    """
    Cell used for showing direction data.
    """

    def __init__(self, content: str, data) -> None:
        """"""
        super().__init__(content, data)

    def set_content(self, content, data) -> None:
        """
        Cell color is set according to direction.
        """
        if str(content) == "Direction.SHORT":
            super().set_content("空", data)
            self.setForeground(COLOR_SHORT)
        elif str(content) == "Direction.LONG":
            super().set_content("多", data)
            self.setForeground(COLOR_LONG)
        else:
            super().set_content("净", data)
            self.setForeground(COLOR_NET)


class BaseMonitor(QtWidgets.QTableWidget):
    """
    Monitor data update.
    """

    event_type: str = ""
    data_key: str = ""
    sorting: bool = False
    headers: dict = {}

    signal: QtCore.Signal = QtCore.Signal(Event)

    def __init__(self, event_engine: EventEngine) -> None:
        """"""
        super().__init__()

        self.event_engine: EventEngine = event_engine
        self.cells: Dict[str, dict] = {}

        self.init_ui()
        self.load_setting()
        self.register_event()

    def init_ui(self) -> None:
        """"""
        self.init_table()
        self.init_menu()

    def init_table(self) -> None:
        """
        Initialize table.
        """
        self.setColumnCount(len(self.headers))

        labels: list = [d["display"] for d in self.headers.values()]
        self.setHorizontalHeaderLabels(labels)

        self.verticalHeader().setVisible(False)
        self.setEditTriggers(self.EditTrigger.NoEditTriggers)
        self.setAlternatingRowColors(True)
        self.setSortingEnabled(self.sorting)

    def init_menu(self) -> None:
        """
        Create right click menu.
        """
        self.menu: QtWidgets.QMenu = QtWidgets.QMenu(self)

        resize_action: QtGui.QAction = QtGui.QAction(_("调整列宽"), self)
        resize_action.triggered.connect(self.resize_columns)
        self.menu.addAction(resize_action)

        save_action: QtGui.QAction = QtGui.QAction(_("保存数据"), self)
        save_action.triggered.connect(self.save_csv)
        self.menu.addAction(save_action)

    def register_event(self) -> None:
        """
        Register event handler into event engine.
        """
        if self.event_type:
            self.signal.connect(self.process_event)
            self.event_engine.register(self.event_type, self.signal.emit)

    def process_event(self, event: Event) -> None:
        """
        Process new data from event and update into table.
        """
        # Disable sorting to prevent unwanted error.
        if self.sorting:
            self.setSortingEnabled(False)

        # Update data into table.
        data = event.data

        if not self.data_key:
            self.insert_new_row(data)
        else:
            key: str = data.__getattribute__(self.data_key)

            if key in self.cells:
                self.update_old_row(data)
            else:
                self.insert_new_row(data)

        # Enable sorting
        if self.sorting:
            self.setSortingEnabled(True)

    def insert_new_row(self, data: Any) -> None:
        """
        Insert a new row at the top of table.
        """
        self.insertRow(0)

        row_cells: dict = {}
        for column, header in enumerate(self.headers.keys()):
            setting: dict = self.headers[header]

            content = data.__getattribute__(header)
            cell: QtWidgets.QTableWidgetItem = setting["cell"](content, data)
            self.setItem(0, column, cell)

            if setting["update"]:
                row_cells[header] = cell

        if self.data_key:
            key: str = data.__getattribute__(self.data_key)
            self.cells[key] = row_cells

    def update_old_row(self, data: Any) -> None:
        """
        Update an old row in table.
        """
        key: str = data.__getattribute__(self.data_key)
        row_cells = self.cells[key]

        for header, cell in row_cells.items():
            content = data.__getattribute__(header)
            cell.set_content(content, data)

    def resize_columns(self) -> None:
        """
        Resize all columns according to contents.
        """
        self.horizontalHeader().resizeSections(
            QtWidgets.QHeaderView.ResizeMode.ResizeToContents
        )

    def save_csv(self) -> None:
        """
        Save table data into a csv file
        """
        path, __ = QtWidgets.QFileDialog.getSaveFileName(
            self, _("保存数据"), "", "CSV(*.csv)"
        )

        if not path:
            return

        with open(path, "w") as f:
            writer = csv.writer(f, lineterminator="\n")

            headers: list = [d["display"] for d in self.headers.values()]
            writer.writerow(headers)

            for row in range(self.rowCount()):
                if self.isRowHidden(row):
                    continue

                row_data: list = []
                for column in range(self.columnCount()):
                    item: QtWidgets.QTableWidgetItem = self.item(row, column)
                    if item:
                        row_data.append(str(item.text()))
                    else:
                        row_data.append("")
                writer.writerow(row_data)

    def contextMenuEvent(self, event: QtGui.QContextMenuEvent) -> None:
        """
        Show menu with right click.
        """
        self.menu.popup(QtGui.QCursor.pos())

    def save_setting(self) -> None:
        """"""
        settings: QtCore.QSettings = QtCore.QSettings(self.__class__.__name__, "custom")
        settings.setValue("column_state", self.horizontalHeader().saveState())

    def load_setting(self) -> None:
        """"""
        settings: QtCore.QSettings = QtCore.QSettings(self.__class__.__name__, "custom")
        column_state = settings.value("column_state")

        if isinstance(column_state, QtCore.QByteArray):
            self.horizontalHeader().restoreState(column_state)
            self.horizontalHeader().setSortIndicator(
                -1, QtCore.Qt.SortOrder.AscendingOrder
            )
