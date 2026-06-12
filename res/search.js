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

function onChangeMedia() {
    const media = document.getElementById("search-media").value;
    document.location = `/search/${media}`;
}

function onChangeEpisode() {
    const media = document.getElementById("search-media").value;
    const episode = document.getElementById("search-episode").value;
    document.location = `/search/${media}/${episode}`;
}

async function generate(media, episode, id) {
    await fetch(`/content/${media}/${episode}/${id}`, { method: "PUT" });
    location.reload();
}