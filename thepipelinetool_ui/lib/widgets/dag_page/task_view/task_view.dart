import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:thepipelinetool/providers/task_view/task_grid.dart';
import 'package:thepipelinetool/widgets/dag_page/task_view/table_cell.dart';
import 'package:two_dimensional_scrollables/two_dimensional_scrollables.dart';

import '../../../classes/selected_task.dart';
import '../../../providers/drawer/selected_task.dart';
// import 'multiplication_table.dart';

class TaskView extends ConsumerStatefulWidget {
  final String dagName;

  const TaskView(this.dagName, {super.key});
  @override
  TaskViewState createState() => TaskViewState();
}

class TaskViewState extends ConsumerState<TaskView>
    with TickerProviderStateMixin {
  late final AnimationController _controller = AnimationController(
    duration: const Duration(milliseconds: 300),
    vsync: this,
  )..forward();
  late final Animation<double> _animation = CurvedAnimation(
    parent: _controller,
    curve: Curves.easeIn,
  );
  final _verticalController = ScrollController();

  @override
  void dispose() {
    _controller.dispose();
    _verticalController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final provider = ref.watch(taskGridProvider(widget.dagName));

    return switch (provider) {
      AsyncData(:final value) => FadeTransition(
          opacity: _animation,
          child: TableView.builder(
            verticalDetails:
                ScrollableDetails.vertical(controller: _verticalController),
            cellBuilder: (BuildContext context, TableVicinity vicinity) {
              if (vicinity.column == 0 && vicinity.row == 0) {
                return TableViewCell(child: Container());
              }

              if (vicinity.column == 0) {
                final task = value["tasks"][vicinity.row - 1];

                return TableViewCell(
                    child: MouseRegion(
                  cursor: SystemMouseCursors.click,
                  child: GestureDetector(
                    onTap: () {
                      ref.read(selectedTaskProvider.notifier).state =
                          SelectedTask(
                              runId: "default",
                              taskId: task["id"].toString(),
                              dagName: widget.dagName);
                      Scaffold.of(context).openEndDrawer();
                    },
                    child: Text(
                      "${task["function_name"]}_${task["id"]}",
                      style: TextStyle(height: 1.1),
                    ),
                  ),
                ));
              }

              var keys = value["runs"].keys.toList();

              if (vicinity.row == 0) {
                return TableViewCell(
                    child: Container(
                  width: outerCellHeight,
                  height: outerCellHeight,
                  alignment: Alignment.center,
                  child: Tooltip(
                    message:
                        "Run Id: ${keys[vicinity.column - 1]}\nDate: ${value["runs"][keys[vicinity.column - 1]]["date"]}",
                    preferBelow: false,
                    verticalOffset: outerCellHeight,
                    showDuration: Duration.zero,
                    child: Container(
                      width: cellWidth,
                      height: cellWidth,
                      decoration: BoxDecoration(
                        color: Colors.green, // TODO
                        borderRadius: BorderRadius.circular(50),
                      ),
                    ),
                  ),
                ));
              }

              final task = value["tasks"][vicinity.row - 1];
              final key = '${task["function_name"]}_${task["id"]}';
              var runId = keys[vicinity.column - 1];
              final containsFunction =
                  value["runs"][runId]["tasks"].containsKey(key);

              if (containsFunction) {
                return TableViewCell(
                    child: MultiplicationTableCell(
                  dagName: widget.dagName,
                  runId: runId,
                  value: value["runs"][runId]["tasks"][key],
                ));
              }

              return TableViewCell(child: Container());
            },
            pinnedColumnCount: 1,
            columnCount: value["runs"].length + 1,
            columnBuilder: (int index) {
              return TableSpan(
                foregroundDecoration:
                    const TableSpanDecoration(border: TableSpanBorder()),
                extent:
                    FixedTableSpanExtent(index == 0 ? 200 : outerCellHeight),
              );
            },
            pinnedRowCount: 1,
            rowCount: value["tasks"].length + 1,
            rowBuilder: (int index) {
              return const TableSpan(
                backgroundDecoration: TableSpanDecoration(),
                extent: FixedTableSpanExtent(outerCellHeight),
              );
            },
          ),
        ),
      AsyncError() => const Text('Oops, something unexpected happened'),
      _ => Container(),
    };
  }
}
