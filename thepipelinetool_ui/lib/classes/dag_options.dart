class DagOptions {
  final bool catchup;
  final DateTime? endDate;
  final int maxAttempts;
  final Duration retryDelay;
  final String? schedule;
  final DateTime? startDate;
  final Duration? timeout;

  DagOptions({
    required this.catchup,
    this.endDate,
    required this.maxAttempts,
    required this.retryDelay,
    this.schedule,
    this.startDate,
    this.timeout,
  });

  factory DagOptions.fromJson(Map<String, dynamic> json) {
    return DagOptions(
      catchup: json['catchup'] ?? false,
      endDate:
          json['end_date'] != null ? DateTime.tryParse(json['end_date']) : null,
      maxAttempts: json['max_attempts'] ?? 1,
      retryDelay: Duration(
        seconds: json['retry_delay']['secs'] ?? 0,
        // nanoseconds: json['retry_delay']['nanos'] ?? 0,
      ),
      schedule: json['schedule'],
      startDate: json['start_date'] != null
          ? DateTime.tryParse(json['start_date'])
          : null,
      timeout: json['timeout'] != null
          ? Duration(seconds: json['timeout']['secs'] ?? 0
              // , nanoseconds: json['timeout']['nanos'] ?? 0
              )
          : null,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'catchup': catchup,
      'end_date': endDate?.toIso8601String(),
      'max_attempts': maxAttempts,
      'retry_delay': {
        'secs': retryDelay.inSeconds,
        'nanos': retryDelay.inMicroseconds % Duration.microsecondsPerSecond,
      },
      'schedule': schedule,
      'start_date': startDate?.toIso8601String(),
      'timeout': timeout != null
          ? {
              'secs': timeout!.inSeconds,
              'nanos': timeout!.inMicroseconds % Duration.microsecondsPerSecond,
            }
          : null,
    };
  }
}
