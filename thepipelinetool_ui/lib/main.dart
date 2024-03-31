// Copyright 2013 The Flutter Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:thepipelinetool/widgets/dag_page/dag_page.dart';

import 'providers/darkmode.dart';
import 'widgets/homescreen.dart';

class Config {
  static const BASE_URL = "http://localhost:8000";
  static const ALL_RUNS = "/runs/all/";
  static const DAGS = "/dags";
  static const RESULTS = "/results/";
  static const RUNS = "/runs/";
  static const TASKS = "/tasks/";
  static const DEFAULT_TASKS = "/tasks/default/";
  static const STATUSES = "/statuses/";
  static const GRAPHS = "/graphs/";
  static const DEFAULT_GRAPHS = "/graphs/default/";
  static const TRIGGER = "/trigger/";
}

/// This sample app shows an app with two screens.
///
/// The first route '/' is mapped to [HomeScreen], and the second route
/// '/details' is mapped to [DetailsScreen].
///
/// The buttons use context.go() to navigate to each destination. On mobile
/// devices, each destination is deep-linkable and on the web, can be navigated
/// to using the address bar.
void main() => runApp(const ProviderScope(child: MyApp()));

CustomTransitionPage<void> pageBuilder(
    BuildContext context, GoRouterState state, Widget widget) {
  return CustomTransitionPage<void>(
    key: state.pageKey,
    child: widget,
    transitionDuration: const Duration(milliseconds: 150),
    transitionsBuilder: (BuildContext context, Animation<double> animation,
        Animation<double> secondaryAnimation, Widget child) {
      // Change the opacity of the screen using a Curve based on the the animation's
      // value
      return FadeTransition(
        opacity: CurveTween(curve: Curves.easeInOut).animate(animation),
        child: child,
      );
    },
  );
}

/// The route configuration.
final GoRouter _router = GoRouter(
  routes: <RouteBase>[
    GoRoute(
      path: '/',
      pageBuilder: (BuildContext context, GoRouterState state) =>
          pageBuilder(context, state, const HomeScreen()),
    ),
    GoRoute(
      name: 'dag',
      path: '/dag/:dag_name',
      pageBuilder: (BuildContext context, GoRouterState state) => pageBuilder(
          context,
          state,
          DetailsPage(dagName: state.pathParameters['dag_name']!)),
    ),
  ],
);

/// The main app.
class MyApp extends ConsumerWidget {
  /// Constructs a [MyApp]
  const MyApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final isDarkTheme = ref.watch(darkmodeProvider);

    return MaterialApp.router(
      routerConfig: _router,
      debugShowCheckedModeBanner: false,
      theme: ThemeData(
        applyElevationOverlayColor: false,
        splashColor: Colors.transparent,
        highlightColor: Colors.transparent,
        hoverColor: Colors.transparent,
        // primarySwatch: Colors.blue,
        visualDensity: VisualDensity.adaptivePlatformDensity,

        primarySwatch: Colors.red,
        primaryColor: isDarkTheme ? Colors.black : Colors.white,

        // backgroundColor: isDarkTheme ? Colors.black : Color(0xffF1F5FB),

        // indicatorColor: isDarkTheme ? const Color(0xff0E1D36) : const Color(0xffCBDCF8),
        indicatorColor: Colors.transparent,
        hintColor: Colors.transparent,
        // buttonColor: isDarkTheme ? Color(0xff3B3B3B) : Color(0xffF1F5FB),

        // hintColor: isDarkTheme ? const Color(0xff280C0B) : const Color(0xffEECED3),

        // highlightColor: isDarkTheme ? Color(0xff372901) : Color(0xffFCE192),
        // hoverColor: isDarkTheme ? Color(0xff3A3A3B) : Color(0xff4285F4),

        // focusColor: isDarkTheme ? const Color(0xff0B2512) : const Color(0xffA8DAB5),
        focusColor: Colors.transparent,
        disabledColor: Colors.grey,
        // textSelectionColor: isDarkTheme ? Colors.white : Colors.black,
        cardColor: isDarkTheme ? const Color(0xFF151515) : Colors.white,
        canvasColor: isDarkTheme ? Colors.black : Colors.grey[50],
        brightness: isDarkTheme ? Brightness.dark : Brightness.light,
        buttonTheme: Theme.of(context).buttonTheme.copyWith(
            colorScheme: isDarkTheme
                ? const ColorScheme.dark()
                : const ColorScheme.light()),
        appBarTheme: const AppBarTheme(
          elevation: 0.0,
        ),
      ),
    );
  }
}
