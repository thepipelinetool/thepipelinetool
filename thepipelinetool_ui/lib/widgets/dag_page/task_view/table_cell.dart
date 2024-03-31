import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:thepipelinetool/classes/selected_task.dart';
import 'package:thepipelinetool/providers/drawer/selected_task.dart';
import 'package:thepipelinetool/classes/task_status.dart';

import '../../../classes/colors.dart';
import '../../../providers/task_status.dart';
import '../../../providers/tooltip.dart';

const double outerCellHeight = 16;
const double cellWidth = 10;
const double firstCellWidth = 100;

class MultiplicationTableCell extends ConsumerWidget {
  final String dagName;
  final String runId;
  final Map value;
  // final Color color;
  //final GlobalKey<ScaffoldState> scaffoldKey;

  const MultiplicationTableCell({
    super.key,
    // required this.scaffoldKey,
    required this.value,
    required this.runId,
    required this.dagName,
    // required this.color,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final taskStatus =
        ref.watch(fetchTaskStatusProvider((runId, value["id"], true)));

    // final vals = <String>[];
    var vals = '';

    var color = Colors.transparent;
    var status = TaskStatus.None;

    switch (taskStatus) {
      case AsyncData(:final value):
        color = getColorByStatus(value);
        status = value;
      case AsyncError():
        ref.invalidate(fetchTaskStatusProvider((runId, value["id"], true)));
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

    return Container(
      width: outerCellHeight,
      height: outerCellHeight,
      decoration: const BoxDecoration(
        //   // color:
        //   //     // value["status"] == "Pending"
        //   //     //     ? switch (taskStatus) {
        //   //     //         AsyncData(:final value) => (){
        //   //     //           print(value);
        //   //     //           return Colors.red;
        //   //     //           return getStylingForGridStatus(value["status"]);}(),
        //   //     //         (_) => Colors.transparent,
        //   //     //       }
        //   //     //     : getStylingForGridStatus(value["status"]),

        //   //     Colors.red,
        // decoration: BoxDecoration(
        border: Border(
          // top: BorderSide(width: 1.0, color: Colors.transparent),
          bottom:
              BorderSide(width: 1.0, color: Color.fromRGBO(158, 158, 158, 1)),
        ),
      ),
      // ),
      alignment: Alignment.center,
      child:

          // Padding(
          //   padding: const EdgeInsets.all(2),
          // child:
          Center(
        child: MouseRegion(
          onEnter: (_) {
            // print(1);
            ref.read(hoveredTooltipProvider.notifier).state = SelectedTask(
                dagName: dagName, runId: runId, taskId: value["id"].toString());
          },
          cursor: SystemMouseCursors.click,
          child: GestureDetector(
            onTap: () {
              // print(runId);
              // print(value);
              ref.read(selectedTaskProvider.notifier).state = SelectedTask(
                  runId: runId,
                  taskId: value["id"].toString(),
                  dagName: dagName);
              // ref.invalidate(fetchTaskStatusProvider(
              //     (dagName, runId, value["id"], false)));
              Scaffold.of(context).openEndDrawer();

              // scaffoldKey.currentState!.openEndDrawer();
            },
            child: vals.isEmpty
                ? Container(
                    width: cellWidth,
                    height: cellWidth,
                    decoration: BoxDecoration(
                      color: color,
                      // borderRadius: BorderRadius.circular(1.5),
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
                          text: "${value["function_name"]}_${value["id"]}\n",
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
                    child: Container(
                      width: cellWidth,
                      height: cellWidth,
                      decoration: BoxDecoration(
                        color: color,
                        borderRadius: BorderRadius.circular(1.5),
                      ),
                    ),
                  ),
          ),
        ),
        // Text(
        //   '${value ?? ''}',
        //   style: TextStyle(fontSize: 16.0),
        // ),
      ),
    );
  }
}
