import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:graphite/graphite.dart';
import 'package:thepipelinetool/classes/selected_task.dart';
import 'package:thepipelinetool/providers/drawer/selected_task.dart';
import 'package:thepipelinetool/providers/graph_view/runs.dart';

import 'node_card.dart';
// import '../main.dart';

// extension CacheForExtension on AutoDisposeRef<Object?> {
//   /// Keeps the provider alive for [duration].
//   void cacheFor(Duration duration) {
//     // Immediately prevent the state from getting destroyed.
//     final link = keepAlive();
//     // After duration has elapsed, we re-enable automatic disposal.
//     final timer = Timer(duration, link.close);

//     // Optional: when the provider is recomputed (such as with ref.watch),
//     // we cancel the pending timer.
//     onDispose(timer.cancel);
//   }
// }

class GraphView extends ConsumerStatefulWidget {
  final String dagName;
  //final GlobalKey<ScaffoldState> scaffoldKey;

  const GraphView({super.key, required this.dagName});

  @override
  GraphViewState createState() => GraphViewState();

  static final p = Paint()
    ..color = Colors.blueGrey
    ..style = PaintingStyle.stroke
    ..strokeCap = StrokeCap.round
    ..strokeJoin = StrokeJoin.round
    ..strokeWidth = 2;
}

class GraphViewState extends ConsumerState<GraphView>
    with TickerProviderStateMixin {
  @override
  void dispose() {
    _controller.dispose();
    _controller2.dispose();

    super.dispose();
  }
  // GraphViewState(this.dagName, {required this.scaffoldKey});

  late final AnimationController _controller = AnimationController(
    duration: const Duration(milliseconds: 300),
    vsync: this,
  )..forward();
  late final Animation<double> _animation = CurvedAnimation(
    parent: _controller,
    curve: Curves.easeIn,
  );

  late final AnimationController _controller2 = AnimationController(
    duration: const Duration(milliseconds: 300),
    vsync: this,
  )..forward();
  late final Animation<double> _animation2 = CurvedAnimation(
    parent: _controller2,
    curve: Curves.easeIn,
  );

  @override
  Widget build(BuildContext context) {
    final runs = ref.watch(runsProvider(widget.dagName));
    final graph = ref.watch(graphProvider(widget.dagName));

    return FadeTransition(
      opacity: _animation,
      child: Column(
        children: [
          Align(
            alignment: Alignment.centerLeft,
            child: DropdownButton<Run>(
              value: ref.watch(selectedRunDropDownProvider(widget.dagName)),
              items: (switch (runs) {
                AsyncData(:final value) => value,
                (_) => [Run.defaultRun]
              })
                  .map<DropdownMenuItem<Run>>(
                (Run value) {
                  return DropdownMenuItem<Run>(
                    value: value,
                    child: Text(value.date),
                  );
                },
              ).toList(),
              onChanged: (Run? newValue) {
                ref
                    .read(selectedRunDropDownProvider(widget.dagName).notifier)
                    .state = newValue!;
                FocusScope.of(context).requestFocus(FocusNode());
              },
            ),
          ),
          Expanded(
              child: switch (graph) {
            AsyncData(:final value) => () {
                final (graph, run) = value;

                final list = graph.map((m) => NodeInput.fromJson(m)).toList();

                final map = {};
                for (final json in graph) {
                  map[json["id"]] = json;
                }

                return FadeTransition(
                  opacity: _animation2,
                  child: InteractiveViewer(
                    minScale: 0.3,
                    boundaryMargin: const EdgeInsets.all(double.infinity),
                    constrained: false,
                    child: DirectGraph(
                      list: list,
                      defaultCellSize: const Size(154.0, 104.0 / 2),
                      cellPadding: const EdgeInsets.all(14),
                      contactEdgesDistance: 5.0,
                      orientation: MatrixOrientation.Horizontal,
                      centered: true,
                      onEdgeTapDown: (details, edge) {
                        // print("${edge.from.id}->${edge.to.id}");
                      },
                      nodeBuilder: (ctx, node) {
                        return NodeCard(
                            dagName: widget.dagName, info: map[node.id]);
                      },
                      paintBuilder: (edge) {
                        return GraphView.p;
                      },
                      onNodeTapUp: (_, node, __) {
                        ref.read(selectedTaskProvider.notifier).state =
                            SelectedTask(
                                runId: run.runId,
                                taskId: node.id,
                                dagName: widget.dagName);
                        Scaffold.of(context).openEndDrawer();
                      },
                    ),
                  ),
                );
              }(),
            AsyncError() => const Text('Oops, something unexpected happened'),
            _ => Container()
            // const Center(child: CircularProgressIndicator())
          })
        ],
      ),
    );
  }
}
