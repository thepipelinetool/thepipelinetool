import 'package:flutter/widgets.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:thepipelinetool/classes/colors.dart';
import 'package:thepipelinetool/providers/task_status.dart';

class Status extends ConsumerWidget {
  final String runId;
  final int taskId;

  const Status(this.runId, this.taskId, {super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final status = ref.watch(fetchTaskStatusProvider((runId, taskId, true)));

    return switch (status) {
      AsyncData(:final value) => Container(
          padding: EdgeInsets.symmetric(horizontal: 10),
          decoration: BoxDecoration(
              border: Border.all(width: 4, color: getColorByStatus(value)),
              borderRadius: BorderRadius.circular(5)),
          child: Center(child: Text(value.toString()))),
      (_) => Container(child: const Center(child: Text('')))
    };
  }
}
