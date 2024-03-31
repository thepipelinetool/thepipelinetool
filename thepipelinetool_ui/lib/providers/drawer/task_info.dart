import 'dart:convert';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:thepipelinetool/providers/drawer/selected_task.dart';

import '../../main.dart';
import '../http_client.dart';

final taskInfoProvider =
    FutureProvider.autoDispose<Map<String, dynamic>>((ref) async {
  final selectedTask = ref.watch(selectedTaskProvider)!;

  var path = '${Config.TASKS}${selectedTask.runId}/${selectedTask.taskId}';

  // print(selectedTask.taskId);
  if (selectedTask.runId == "default") {
    path =
        '${Config.DEFAULT_TASKS}${selectedTask.dagName}/${selectedTask.taskId}';
  }
  final client = ref.watch(clientProvider);

  final response = await client.get(Uri.parse('${Config.BASE_URL}$path'));

  // final map = ;

  return jsonDecode(response.body) as Map<String, dynamic>;
});
