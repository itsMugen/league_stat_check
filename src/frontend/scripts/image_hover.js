var list_ids = [];
var list_elements = new Array();

function fill_image(element, imageUrl) {
    element.style.backgroundImage = `url(${imageUrl})`;
}

function remove_image(element) { 
    if (list_elements.includes(element)){
        ;
    } else {
        element.style.backgroundImage = "none";
    }
} 

function fill_onclick(element, imageUrl) {
    if (list_ids.includes(element.id)){
        let index = list_ids.indexOf(element.id);
        let previous_element = list_ids.splice(index, 1);
        let elements_in_dom = document.querySelectorAll("#"+previous_element);
        elements_in_dom.forEach(function(element){
            element.style.backgroundImage = "none";
        });
    }

    list_ids.push(element.id);
    element.style.backgroundImage = `url(${imageUrl})`;

    let new_stat = true;
    list_elements.forEach(stored_element => {
        if (stored_element.id == element.id){
            let index =  list_elements.indexOf(stored_element);
            list_elements.splice(index, 1);
            new_stat = false;
        }
    }); 

    if (new_stat){
        list_elements.push(element);
    } else {
        list_elements.push(element);
    }
}

function activate_submit(){
    if (list_elements.length == 9){
        let button = document.getElementById("check_them_stats");
        button.style.display = "flex";
    }

}

function purge() {
    list_elements = new Array();
    list_ids = new Array();
}
