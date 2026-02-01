interface Tab {
  id: string;
  title: string;
  type: 'logs' | 'edit' | 'custom';
  data: any;
  onClose?: () => void;
}

class BottomDrawerStore {
  open = $state(false);
  tabs = $state<Tab[]>([]);
  activeTabId = $state<string | null>(null);

  openTab(tab: Tab) {
    // Check if tab already exists
    const existing = this.tabs.find(t => t.id === tab.id);
    if (existing) {
      // Just activate it
      this.activeTabId = tab.id;
      this.open = true;
      return;
    }

    // Add new tab
    this.tabs.push(tab);
    this.activeTabId = tab.id;
    this.open = true;
  }

  closeTab(tabId: string) {
    const tab = this.tabs.find(t => t.id === tabId);
    if (tab?.onClose) {
      tab.onClose();
    }

    this.tabs = this.tabs.filter(t => t.id !== tabId);

    // If we closed the active tab, activate another one
    if (this.activeTabId === tabId) {
      if (this.tabs.length > 0) {
        this.activeTabId = this.tabs[0].id;
      } else {
        this.activeTabId = null;
        this.open = false;
      }
    }
  }

  setActiveTab(tabId: string) {
    if (this.tabs.find(t => t.id === tabId)) {
      this.activeTabId = tabId;
    }
  }

  close() {
    this.open = false;
  }

  toggle() {
    this.open = !this.open;
  }

  get activeTab(): Tab | null {
    if (!this.activeTabId) return null;
    return this.tabs.find(t => t.id === this.activeTabId) || null;
  }
}

export const bottomDrawerStore = new BottomDrawerStore();
