import 'dart:async';
import 'dart:io';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:retry/retry.dart';

import '../classes/task_status.dart';
import '../main.dart';
import '../views/task_view/http_client_provider.dart';

final fetchTaskStatusProvider = FutureProvider.autoDispose
    .family<TaskStatus, (String?, int?, bool)>((ref, args) async {
  // final runId = ref.watch(selectedItemProvider(dagName));

  final (runId, taskId, refresh) = args;

  var path = Uri.parse('${Config.BASE_URL}${Config.STATUSES}$runId/$taskId');
  // final client = http.Client();
  // ref.onDispose(client.close);
  final client = ref.watch(clientProvider);

  final response = await retry(
    // Make a GET request
    () async => await client.get(path),
    // Retry on SocketException or TimeoutException
    retryIf: (e) => e is SocketException || e is TimeoutException,
  );
  // final response = ;
  ref.keepAlive();
  final map = response.bodyBytes.first.toTaskStatus();
  // print('MAP $map');

  // final map = jsonDecode(response.) as Map<String, dynamic>;
  // print("map ${map}");

  if (refresh &&
      {TaskStatus.Pending, TaskStatus.Running, TaskStatus.Retrying}
          .contains(map)) {
    Timer.periodic(const Duration(seconds: 3), (t) async {
      final response2 = await client.get(path);
      final map2 = response2.bodyBytes.first.toTaskStatus();
      // final map2 = jsonDecode(response2.body) as Map<String, dynamic>;
      if (map2 != map) {
        // print('refresh');
        t.cancel();
        ref.invalidateSelf();
      }
    });
  }

  return map;
  // return {"status": "Pending"};
});
