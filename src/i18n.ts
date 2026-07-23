const zhCN = {
  appName: "See See",
  loading: "正在加载…",
  openSettings: "模型设置",
  openPrompts: "提示词",
  openHistory: "历史记录",
  quit: "退出",
  retry: "重试",
  cancel: "取消",
  empty: "暂无内容",
  unknownError: "发生了未知错误",
} as const;

export type MessageKey = keyof typeof zhCN;

export function t(key: MessageKey): string {
  return zhCN[key];
}
