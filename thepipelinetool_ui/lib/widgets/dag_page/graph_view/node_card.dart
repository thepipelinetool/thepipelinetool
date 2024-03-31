import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:thepipelinetool/classes/selected_task.dart';
import 'package:thepipelinetool/classes/task_status.dart';

import '../../../classes/colors.dart';
import '../../../providers/graph_view/runs.dart';
import '../../../providers/task_status.dart';
import '../../../providers/tooltip.dart';
import '../task_view/table_cell.dart';
import 'graph_view.dart';

class NodeCard extends ConsumerWidget {
  final String dagName;
  // final String runId;
  final Map info;

  const NodeCard(
      {super.key,
      required this.dagName,
      // required this.runId,
      required this.info});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final run = ref.watch(selectedRunDropDownProvider(dagName));
    var vals = '';

    var color = Colors.grey.shade400;

    if (run.runId != "default") {
      // print((dagName.runtimeType, runId.runtimeType, info["id"].runtimeType));
      final taskStatus = ref.watch(
          fetchTaskStatusProvider((run.runId, int.parse(info["id"]), true)));

      var status = TaskStatus.None;

      switch (taskStatus) {
        case AsyncData(:final value):
          status = value;
          color = getColorByStatus(value);
        // return getStylingForGridStatus(value["status"]);
      }

      final tooltip = ref.watch(fetchTooltip(status));

      switch (tooltip) {
        case AsyncData(:final value):
          vals += value;
        // case AsyncError():
        //   ref.invalidate(fetchTooltip);

        // ref.invalidate(
        //     fetchTaskStatusProvider((dagName, runId, value["id"], true)));
        // return getStylingForGridStatus(value["status"]);
      }
    }

    return
        // Card(
        //   clipBehavior: Clip.hardEdge,
        //   child:
        MouseRegion(
      onEnter: (_) {
        // print(1);
        ref.read(hoveredTooltipProvider.notifier).state = SelectedTask(
            dagName: dagName, runId: run.runId, taskId: info["id"].toString());
      },
      cursor: SystemMouseCursors.click,
      child: Container(
        decoration: BoxDecoration(
            border: Border.all(
              color: color,
              width: 4.0,
            ),
            color: Colors.white,
            borderRadius: BorderRadius.circular(4)),
        child: vals.isEmpty
            ? Center(
                child: Text(
                  info["name"],
                  style: const TextStyle(fontSize: 20.0),
                ),
              )
            : Tooltip(
                // height: vals.isEmpty ? 0 : null,
                // message: 'I am a Tooltip',
                richMessage: TextSpan(
                  // text: 'I am a rich tooltip. ',
                  // text: '${value["function_name"]}_${value["id"]}\n$vals',
                  // style: TextStyle(color: Colors.red),
                  children: <InlineSpan>[
                    TextSpan(
                      text: "${info["name"]}\n",
                      style: const TextStyle(fontWeight: FontWeight.bold),
                    ),
                    TextSpan(
                      text: vals,
                      // style: const TextStyle(fontWeight: FontWeight.bold),
                    ),
                  ],
                ),
                // onTriggered: () {
                // },
                preferBelow: false,
                verticalOffset: outerCellHeight,
                showDuration: Duration.zero,
                child: Center(
                  child: Text(
                    info["name"],
                    style: const TextStyle(fontSize: 20.0),
                  ),
                ),
              ),
      ),
    );
  }
}
