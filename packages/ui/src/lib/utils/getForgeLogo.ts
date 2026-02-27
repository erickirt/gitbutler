import { type NewIconName } from "$lib/icons/names";

export function getForgeLogo(forgeName: string, small = false): NewIconName {
	if (forgeName === "gitlab") {
		if (small) {
			return "gitlab";
		}
		return "gitlab";
	} else if (forgeName === "github") {
		if (small) {
			return "github";
		}

		return "github";
	}

	return "question";
}
