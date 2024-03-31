import 'package:thepipelinetool/classes/dag_options.dart';

class DagInfo {
  final String dagName;
  final DateTime? lastRun;
  final DateTime? nextRun;
  final DagOptions options;

  DagInfo({
    required this.dagName,
    this.lastRun,
    this.nextRun,
    required this.options,
  });

  factory DagInfo.fromJson(Map<String, dynamic> json) => DagInfo(
      dagName: json['dag_name'],
      lastRun: (json['last_run'] as List<dynamic>).isNotEmpty
          ? DateTime.tryParse(json['last_run'][0]["date"])
          : null,
      nextRun: (json['next_run'] as List<dynamic>).isNotEmpty
          ? DateTime.tryParse(json['next_run'][0]["date"])
          : null,
      options: DagOptions.fromJson(json['options']));
}
