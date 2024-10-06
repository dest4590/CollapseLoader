from collapse.modules.utils.Language import lang


def test_lang():
    lang.set_language('en')
    
    assert lang.t('menu.return') == 'Return' 
    assert lang.t('test.test') == 'test.test'