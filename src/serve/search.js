function onSubmit() {
    const form = document.getElementById("search-form");
    form.action = "/search";
    const media = document.getElementById("search-media").value;
    const episode = document.getElementById("search-episode").value;
    if (media) {
        form.action += `/${media}`;
    }
    if (episode) {
        form.action += `/${episode}`;
    }
}

async function onChangeMedia() {
    const episodeSelect = document.getElementById("search-episode");
    episodeSelect.innerHTML = `\<option value="" selected\>(any)\</option\>`;
    const media = document.getElementById("search-media").value;
    if (media) {
        const episodes = await (await fetch(`/list/${media}`)).json();
        console.log(episodeSelect, episodes);
        for (const episode of episodes) {
            episodeSelect.innerHTML += `\n<option value="${episode}">${episode}</option>`;
        }   
    }
}

window.onload = onChangeMedia();