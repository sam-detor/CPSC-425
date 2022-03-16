/*
 * i2s-beep.c
 *
 * author: Furkan Cayci
 * description:
 *   talk to CS43L22 Audio DAC over I2C
 *   connected to I2C1 PB6, PB9
 *   setups CS43L22 to play 6 beep sounds from headphone jack
 *   Audio out is connected to I2S3
 */

#include "stm32f4xx.h"
#include "system_stm32f4xx.h"
#include "cs43l22.h"

enum processState {NO_PRESS, FIRST_PRESS, FIRST_PRESS_DEBOUNCE, FIRST_UNPRESS,FIRST_UNPRESS_DEBOUNCE, SECOND_PRESS, SECOND_PRESS_DEBOUNCE, LONG_PRESS};
enum processState myState = NO_PRESS;
/*************************************************
* function declarations
*************************************************/
int main(void);

void TIM2_IRQHandler(void) //refresh the charge on PC7 every .3 seconds
{
    if (TIM2->DIER & 0x01) {
        if (TIM2->SR & 0x01) {
            TIM2->SR &= ~(1U << 0);
        }
    }
    if(myState == FIRST_PRESS_DEBOUNCE)
    {
        myState = LONG_PRESS;
        GPIOD->ODR ^= (1 << 15); // toggle blue led
    }
}

void TIM3_IRQHandler(void) //refresh the charge on PC7 every .3 seconds
{
    
    // clear interrupt status
    if (TIM3->DIER & 0x01) {
        if (TIM3->SR & 0x01) {
            TIM3->SR &= ~(1U << 0);
        }
    }
    //GPIOD->ODR ^= (1 << 12);
    if(myState == FIRST_UNPRESS)
    {
        myState = NO_PRESS;
        GPIOD->ODR ^= (1 << 12); // toggle green led
    }
}

void TIM4_IRQHandler(void) //refresh the charge on PC7 every .3 seconds
{
    
    // clear interrupt status
    if (TIM4->DIER & 0x01) {
        if (TIM4->SR & 0x01) {
            TIM4->SR &= ~(1U << 0);
        }
    }
    //GPIOD->ODR ^= (1 << 13);
    if (myState == FIRST_PRESS)
    {
        myState = FIRST_PRESS_DEBOUNCE;
    }
    else if (myState == SECOND_PRESS)
    {
        myState = SECOND_PRESS_DEBOUNCE;
    }
}

void EXTI0_IRQHandler(void)
{

    // Check if the interrupt came from exti0
    if (EXTI->PR & (1 << 0))
    {
        EXTI->PR = (1 << 0); //set a timer, if it's not called before timer is done --> long press
        if(myState == NO_PRESS)
        {
            TIM3->CR1 |= (1 << 0);
            TIM2->CR1 |= (1 << 0);
            TIM4->CR1 |= (1 << 0);
            myState = FIRST_PRESS;
        }
        else if(myState == FIRST_PRESS_DEBOUNCE)
        {
            myState = FIRST_UNPRESS;
            TIM2->CR1 &= ~(1U << 0);
            TIM2->CNT = 0;
        }
        else if (myState == LONG_PRESS)
        {
            myState = NO_PRESS;
        }
        else if (myState == FIRST_UNPRESS)
        {
            GPIOD->ODR ^= (1 << 14);
            myState = SECOND_PRESS;
            TIM4->CR1 |= (1 << 0);
        }
        else if (myState == SECOND_PRESS_DEBOUNCE)
        {
            myState = NO_PRESS;
        }
        
    }

    return;
}

/*************************************************
* main code starts from here
*************************************************/
int main(void)
{
    /* set system clock to 168 Mhz */
    set_sysclk_to_100();

    //*******************************
    // setup LEDs - GPIOD 12,13,14,15
    //*******************************
    RCC->AHB1ENR |= RCC_AHB1ENR_GPIODEN;
    GPIOD->MODER &= ~(0xFFU << 24);
    GPIOD->MODER |= (0x55 << 24);
    GPIOD->ODR    = 0x0000;

    // TIM2 SET UP
    // enable SYSCFG clock (APB2ENR: bit 14)
    RCC->APB2ENR |= (1 << 14);

    //Button set up
    
    // enable GPIOA clock (AHB1ENR: bit 0)
    RCC->AHB1ENR |= (1 << 0);
    GPIOA->MODER &= 0xFFFFFFFC;   // Reset bits 0-1 to clear old values
    GPIOA->MODER |= 0x00000000;   // Make button an input

    SYSCFG->EXTICR[0] |= 0x00000000; // Write 0000 to map PA0 to EXTI0

    EXTI->RTSR |= 0x00001;   // Enable rising edge trigger on EXTI0 (tells you when button is pressed)
    EXTI->FTSR |= 0x00001;
     // Mask the used external interrupt numbers.
    EXTI->IMR |= 0x00001;    // Mask EXTI0

    //TIM3 Set up
    // enable TIM3 clock (bit1)
    RCC->APB1ENR |= (1 << 0);

    TIM2->PSC = 4999; //set TIM3 prescalar
    TIM2->ARR = 11000; //set auto refil value to 1.1 seconds
    TIM2->CR1 |= (1 << 3);
    TIM2->DIER |= (1 << 0);  //enable TIM3 interrupt
    TIM2->CR1 |= (1 << 2);
    
    //TIM3 Set up
    // enable TIM3 clock (bit1)
    RCC->APB1ENR |= (1 << 1);

    TIM3->PSC = 4999; //set TIM3 prescalar
    TIM3->ARR = 3000; //set auto refil value to 0.3 seconds
    TIM3->CR1 |= (1 << 3);
    TIM3->DIER |= (1 << 0);  //enable TIM3 interrupt
    TIM3->CR1 |= (1 << 2);

    RCC->APB1ENR |= (1 << 2);

    TIM4->PSC = 4999; //set TIM3 prescalar
    TIM4->ARR = 150; //set auto refil value to 0.3 seconds
    TIM4->DIER |= (1 << 0);  //enable TIM3 interrupt
    TIM4->CR1 |= (1 << 3);
    TIM4->CR1 |= (1 << 2);

    // Set Priority for each interrupt request
    NVIC_SetPriority(EXTI0_IRQn, 1); // Priority level 1
    // enable EXT0 IRQ from NVIC
    NVIC_EnableIRQ(EXTI0_IRQn);

    NVIC_SetPriority(TIM2_IRQn, 2); // Priority level 2
    // enable TIM2 IRQ from NVIC
    NVIC_EnableIRQ(TIM2_IRQn);

    NVIC_SetPriority(TIM3_IRQn, 3); // Priority level 2
    // enable TIM3 IRQ from NVIC
    NVIC_EnableIRQ(TIM3_IRQn);

    NVIC_SetPriority(TIM4_IRQn, 3); // Priority level 2
    // enable TIM3 IRQ from NVIC
    NVIC_EnableIRQ(TIM4_IRQn);
    //TIM3->CR1 |= (1 << 0);
    GPIOD->ODR ^= (1 << 13);
    TIM3->CR1 |= (1 << 0);
    TIM2->CR1 |= (1 << 0);
    TIM4->CR1 |= (1 << 0);
    
    while(1)
    {
    }
    return 0;
}
