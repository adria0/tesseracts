$(function () {
    $( '.searchTerm' ).bind('keypress', function(e){
        if ( e.keyCode == 13 ) {
        window.location.href = "/"+$('.searchTerm').val()
        }
    });
    $('.searchButton').on('click',function(){
        window.location.href = "/"+$('.searchTerm').val()
    });
});
