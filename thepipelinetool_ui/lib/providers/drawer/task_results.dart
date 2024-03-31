import 'dart:convert';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:thepipelinetool/providers/drawer/selected_task.dart';

import '../../main.dart';
import '../http_client.dart';

final taskResultsProvider =
    FutureProvider.autoDispose<List<Map<String, dynamic>>>((ref) async {
  final selectedTask = ref.watch(selectedTaskProvider)!;

  final client = ref.watch(clientProvider);

  final response = await client.get(Uri.parse(
      '${Config.BASE_URL}${Config.RESULTS}${selectedTask.runId}/${selectedTask.taskId}'));

  return (jsonDecode(response.body) as List).cast<Map<String, dynamic>>();
});
