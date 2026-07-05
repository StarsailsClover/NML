import 'package:flutter/material.dart';

import 'nml_core.dart';

void main() {
  runApp(const NMLApp());
}

class NMLApp extends StatelessWidget {
  const NMLApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'NML',
      debugShowCheckedModeBanner: false,
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(
          seedColor: const Color(0xFF4F8CFF),
          brightness: Brightness.dark,
        ),
        useMaterial3: true,
      ),
      home: const HomePage(),
    );
  }
}

class HomePage extends StatefulWidget {
  const HomePage({super.key});

  @override
  State<HomePage> createState() => _HomePageState();
}

class _HomePageState extends State<HomePage> {
  NMLCore? _core;
  String _status = '正在初始化 NML Core...';
  String _coreVersion = 'unknown';
  final _playerController = TextEditingController(text: 'Player');
  final _versionController = TextEditingController(text: '1.16.5');

  @override
  void initState() {
    super.initState();
    _initCore();
  }

  void _initCore() {
    try {
      final core = NMLCore.load();
      final ok = core.init();
      setState(() {
        _core = ok ? core : null;
        _coreVersion = ok ? core.version() : 'unknown';
        _status = ok ? 'NML Core 已就绪' : 'NML Core 初始化失败';
      });
    } catch (e) {
      setState(() {
        _status = '无法加载 nml_core.dll：$e';
      });
    }
  }

  void _launch() {
    final core = _core;
    if (core == null) {
      setState(() => _status = 'NML Core 未初始化');
      return;
    }

    final code = core.launch(
      versionId: _versionController.text.trim(),
      playerName: _playerController.text.trim(),
      offline: true,
    );

    setState(() {
      _status = code == 0 ? '启动命令已发送' : '启动失败，错误码：$code';
    });
  }

  @override
  void dispose() {
    _core?.dispose();
    _playerController.dispose();
    _versionController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Row(
        children: [
          NavigationRail(
            selectedIndex: 0,
            labelType: NavigationRailLabelType.all,
            destinations: const [
              NavigationRailDestination(icon: Icon(Icons.home_outlined), selectedIcon: Icon(Icons.home), label: Text('主页')),
              NavigationRailDestination(icon: Icon(Icons.download_outlined), selectedIcon: Icon(Icons.download), label: Text('下载')),
              NavigationRailDestination(icon: Icon(Icons.people_outline), selectedIcon: Icon(Icons.people), label: Text('联机')),
              NavigationRailDestination(icon: Icon(Icons.settings_outlined), selectedIcon: Icon(Icons.settings), label: Text('设置')),
            ],
          ),
          const VerticalDivider(width: 1),
          Expanded(
            child: Padding(
              padding: const EdgeInsets.all(32),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text('N0th1ngness Minecraft Launcher', style: Theme.of(context).textTheme.headlineMedium),
                  const SizedBox(height: 8),
                  Text('Core $_coreVersion · $_status', style: Theme.of(context).textTheme.bodyMedium),
                  const SizedBox(height: 32),
                  Card(
                    child: Padding(
                      padding: const EdgeInsets.all(24),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text('快速启动', style: Theme.of(context).textTheme.titleLarge),
                          const SizedBox(height: 16),
                          TextField(
                            controller: _versionController,
                            decoration: const InputDecoration(labelText: 'Minecraft 版本', border: OutlineInputBorder()),
                          ),
                          const SizedBox(height: 12),
                          TextField(
                            controller: _playerController,
                            decoration: const InputDecoration(labelText: '离线玩家名', border: OutlineInputBorder()),
                          ),
                          const SizedBox(height: 16),
                          FilledButton.icon(
                            onPressed: _launch,
                            icon: const Icon(Icons.play_arrow),
                            label: const Text('启动游戏'),
                          ),
                        ],
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }
}
