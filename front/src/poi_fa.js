export default {
	by_kind(wordlist) {
		var all = wordlist.map(w => {
			return this.by_name(w, false);
		}).filter(x => typeof(x) != "undefined");
		if (all.length) {
			return all[0];
		}
		return this.by_name(wordlist[0], true);
	},
	by_name(word, use_default) {
		var ic = this.icons[word];
		if (ic) {
			return ic;
		}
		if (use_default)
			return this.icons['default'];
	},
	'icons': {
		'default': 'mdi-domain',
		'pub': 'mdi-glass-cocktail',
		'bar': 'mdi-glass-cocktail',
		'billiards': 'mdi-billiards',
        'curloc': 'mdi-crosshairs',
        'bicycle': 'mdi-bicycle',
        'toilets': 'mdi-toilet',
        'drinking_water': 'mdi-water-pump',
        'post_box': 'mdi-post',
        'hotel': 'mdi-hotel'
	}
};
