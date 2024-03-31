import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:http/http.dart' as http;

final clientProvider = StateProvider.autoDispose<http.Client>((ref) {
  final client = http.Client();

  ref.keepAlive();
  ref.onDispose(client.close);
  return client;
});
