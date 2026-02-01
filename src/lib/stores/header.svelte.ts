class HeaderStore {
  title = $state("Dashboard");

  setTitle(title: string) {
    this.title = title;
  }
}

export const headerStore = new HeaderStore();
