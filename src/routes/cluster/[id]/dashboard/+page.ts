import { redirect } from "@sveltejs/kit";
import type { PageLoad } from "./$types";

export const load: PageLoad = ({ url }) => {
  const dashboard = url.pathname.replace(/\/dashboard\/?$/, "");
  redirect(308, `${dashboard}${url.search}`);
};
