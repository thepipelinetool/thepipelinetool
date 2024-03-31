import 'dart:convert';
import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:thepipelinetool/providers/http_client.dart';
import '../../main.dart';

// final fetchRunsWithTasksProvider = FutureProvider.family
//     .autoDispose<Map<String, dynamic>, String>((ref, dagName) async {
//   final client = ref.watch(clientProvider);

//   final runsWithTasksResponse = await client.get(
//     Uri.parse('${Config.BASE_URL}runs_with_tasks/$dagName'),
//   );

//   return await compute(jsonDecode, runsWithTasksResponse.body) as Map<String, dynamic>;
// });

// final fetchTasksProvider = FutureProvider.autoDispose
//     .family<List<Map<String, dynamic>>, String>((ref, dagName) async {
//   // final path = '/default_tasks/$dagName';
//   final client = ref.watch(clientProvider);

//   final defaultTasksResponse = await client.get(
//     Uri.parse('${Config.BASE_URL}default_tasks/$dagName'),
//   );

//   return (await compute(jsonDecode, defaultTasksResponse.body) as List<dynamic>)
//       .cast<Map<String, dynamic>>();
// });

final taskGridProvider = FutureProvider.family
    .autoDispose<Map<String, dynamic>, String>((ref, dagName) async {
  final client = ref.watch(clientProvider);

  // final taskProvider = await ref.watch(fetchTasksProvider(dagName).future);
  // final runsProvider =
  //     await ref.watch(fetchRunsWithTasksProvider(dagName).future);

  final runsWithTasksResponse = await client
      .get(Uri.parse('${Config.BASE_URL}${Config.ALL_RUNS}$dagName'));
  final defaultTasksResponse = await client
      .get(Uri.parse('${Config.BASE_URL}${Config.DEFAULT_TASKS}$dagName'));

  return {
    'tasks':
        (await compute(jsonDecode, defaultTasksResponse.body) as List<dynamic>)
            .cast<Map<String, dynamic>>(),
    'runs': await compute(jsonDecode, runsWithTasksResponse.body)
        as Map<String, dynamic>,
  };
});
