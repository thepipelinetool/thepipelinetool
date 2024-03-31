import 'dart:convert';
import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../main.dart';
import '../http_client.dart';

final runsProvider =
    FutureProvider.family.autoDispose<List<Run>, String>((ref, dagName) async {
  final client = ref.watch(clientProvider);

  final response =
      await client.get(Uri.parse('${Config.BASE_URL}${Config.RUNS}$dagName'));

  return (await compute(jsonDecode, response.body) as List<dynamic>)
      .cast<Map>()
      .map((i) => Run.fromJson(i))
      .toList()
      .reversed
      .toList()
    ..add(Run.defaultRun);
});

class Run {
  final String date;
  final String runId;

  Run({required this.date, required this.runId});

  static final defaultRun = Run(date: "default", runId: "default");

  static Run fromJson(Map m) => Run(date: m["date"], runId: m["run_id"]);
}

final selectedRunDropDownProvider =
    StateProvider.family.autoDispose<Run, String>((ref, dagName) {
  final runs = ref.watch(runsProvider(dagName));

  return switch (runs) {
    AsyncData(:final value) => value.first,
    (_) => Run.defaultRun
  };
});

final graphProvider = FutureProvider.autoDispose
    .family<(List<Map<String, dynamic>>, Run), String>((ref, dagName) async {
  final run = ref.watch(selectedRunDropDownProvider(dagName));

  var path = '${Config.GRAPHS}${run.runId}';

  if (run.runId == "default") {
    path = '${Config.DEFAULT_GRAPHS}$dagName';
  }
  final client = ref.watch(clientProvider);

  // final client = http.Client();
  // ref.onDispose(client.close);
  final response = await client.get(Uri.parse('${Config.BASE_URL}$path'));
  ref.keepAlive();

  final map = (await compute(jsonDecode, response.body) as List<dynamic>)
      .cast<Map<String, dynamic>>();
  // print(runId);
  // print(runId != "default");
  if (run.runId != "default" &&
      map.any(
          (m) => {"Pending", "Running", "Retrying"}.contains(m['status']))) {
    Future.delayed(const Duration(seconds: 3), () {
      // print('refresh');
      ref.invalidateSelf();
    });
  }

  return (map, run);
});
