import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:thepipelinetool/providers/drawer/task_info.dart';
import 'package:thepipelinetool/widgets/drawer/attempts.dart';
import 'package:thepipelinetool/widgets/drawer/status.dart';

import '../../providers/drawer/selected_task.dart';
import '../dag_page/dag_page.dart';

JsonEncoder encoder = const JsonEncoder.withIndent('    ');
String prettyprint = encoder.convert(json);

class MyDrawer extends ConsumerStatefulWidget {
  const MyDrawer({super.key});

  @override
  MyDrawerState createState() => MyDrawerState();
}

class MyDrawerState extends ConsumerState<MyDrawer>
    with TickerProviderStateMixin {
  late final AnimationController _controller = AnimationController(
    duration: const Duration(milliseconds: 300),
    vsync: this,
  )..forward();
  late final Animation<double> _animation = CurvedAnimation(
    parent: _controller,
    curve: Curves.easeIn,
  );
  @override
  void dispose() {
    _controller.dispose();

    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    // final appState = ref.watch(selectedTaskProvider);

    final task = ref.watch(taskInfoProvider);
    final runId = ref.watch(selectedTaskProvider)?.runId;
    final isDefault = runId == "default";

    return switch (task) {
      AsyncData(value: final value) => () {
          final items = [
            const SizedBox(height: 10),
            Row(crossAxisAlignment: CrossAxisAlignment.center, children: [
              SizedBox(
                key: Key("${value["function_name"]}_${value["id"]}"),
                child: Text(
                  "${value["function_name"]}_${value["id"]}",
                  style: const TextStyle(fontSize: 20),
                ),
              ),
              const Spacer(),
              if (!isDefault) Status(runId!, value["id"]),
            ]),
            const SizedBox(height: 10),
            SingleChildScrollView(
              child: ExpansionPanelList.radio(
                elevation: 0,
                children: [
                  ExpansionPanelRadio(
                    canTapOnHeader: true,
                    headerBuilder: (BuildContext context, bool isExpanded) =>
                        const ListTile(title: Text('Template Args')),
                    body: Align(
                      alignment: Alignment.topLeft,
                      child: Container(
                        padding: const EdgeInsets.only(
                            left: kHorizontalPadding,
                            right: kHorizontalPadding,
                            bottom: kHorizontalPadding),
                        child: SelectableText(
                          encoder.convert(value["template_args"]),
                        ),
                      ),
                    ),
                    value: 'Template Args',
                  ),
                  if (!isDefault)
                    ExpansionPanelRadio(
                        canTapOnHeader: true,
                        headerBuilder:
                            (BuildContext context, bool isExpanded) =>
                                const ListTile(title: Text('Attempts')),
                        body: const Align(
                            alignment: Alignment.topLeft, child: Attempts()),
                        value: 'Attempts')
                ],
              ),
            )
          ];
          return FadeTransition(
            opacity: _animation,
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 10),
              child: ListView.builder(
                  itemCount: items.length,
                  itemBuilder: (ctx, index) => items[index]),
            ),
          );
        }(),
      (_) => Container()
    };
  }
}
