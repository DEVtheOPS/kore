export interface Bookmark {
  id: string;
  cluster_id: string;
  order: number;
}

const STORAGE_KEY = 'kore-bookmarks';

class BookmarksStore {
  bookmarks = $state<Bookmark[]>([]);

  constructor() {
    this.load();
  }

  load() {
    if (typeof localStorage === 'undefined') return;
    
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved) {
      try {
        this.bookmarks = JSON.parse(saved);
      } catch (e) {
        console.error('Failed to parse bookmarks', e);
        this.bookmarks = [];
      }
    }
  }

  save() {
    if (typeof localStorage === 'undefined') return;
    
    localStorage.setItem(STORAGE_KEY, JSON.stringify(this.bookmarks));
  }

  add(clusterId: string) {
    const maxOrder = this.bookmarks.reduce((max, b) => Math.max(max, b.order), -1);
    const bookmark: Bookmark = {
      id: crypto.randomUUID(),
      cluster_id: clusterId,
      order: maxOrder + 1,
    };
    this.bookmarks.push(bookmark);
    this.save();
  }

  remove(clusterId: string) {
    this.bookmarks = this.bookmarks.filter((b) => b.cluster_id !== clusterId);
    this.save();
  }

  isBookmarked(clusterId: string): boolean {
    return this.bookmarks.some((b) => b.cluster_id === clusterId);
  }

  toggle(clusterId: string) {
    if (this.isBookmarked(clusterId)) {
      this.remove(clusterId);
    } else {
      this.add(clusterId);
    }
  }

  reorder(fromIndex: number, toIndex: number) {
    const item = this.bookmarks[fromIndex];
    this.bookmarks.splice(fromIndex, 1);
    this.bookmarks.splice(toIndex, 0, item);
    
    // Update order values
    this.bookmarks.forEach((b, index) => {
      b.order = index;
    });
    
    this.save();
  }

  getBookmarkedClusterIds(): string[] {
    return this.bookmarks
      .sort((a, b) => a.order - b.order)
      .map((b) => b.cluster_id);
  }
}

export const bookmarksStore = new BookmarksStore();
