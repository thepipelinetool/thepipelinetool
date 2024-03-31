import 'dart:convert';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../classes/dag_info.dart';
import '../main.dart';
import 'http_client.dart';

final dagsProvider = FutureProvider.autoDispose<List<DagInfo>>((ref) async {
  final client = ref.watch(clientProvider);
  final response =
      await client.get(Uri.parse('${Config.BASE_URL}${Config.DAGS}'));

  return (jsonDecode(response.body) as List<dynamic>)
      .cast<Map<String, dynamic>>()
      .map((e) => DagInfo.fromJson(e))
      .toList(growable: false);
});
